//! Multiplexed TCP accept, handshake, and command scheduling (S02–S06).
//!
//! Non-blocking `mio` poll loop: accept clients, send [`WELCOME`], complete the
//! subject handshake (RQ19 / AQ14), spawn players, and run the per-player
//! command queue with `t`-based delays (RQ10–RQ12).

use std::collections::HashMap;
use std::io::{self, ErrorKind, Read, Write};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Token};

use crate::cli::Config;
use crate::commands::{self, Command, KO};
use crate::player::PlayerSet;
use crate::world::{SeededRng, World};

/// Exact welcome line from the subject handshake table (`WELCOME\n`).
pub const WELCOME: &[u8] = b"WELCOME\n";

/// Cap on a single inbound protocol line (team name / later commands).
const MAX_LINE_BYTES: usize = 1024;

const LISTENER: Token = Token(0);
/// Poll wake interval so the loop never blocks forever (RQ16) and tests can stop.
const POLL_TIMEOUT: Duration = Duration::from_millis(50);

/// Per-team free connection slots (`-c` at start).
#[derive(Debug, Clone)]
struct TeamSlots {
    /// team name → (free, max)
    inner: HashMap<String, (u32, u32)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum JoinError {
    Unknown,
    Full,
}

impl TeamSlots {
    fn new(teams: &[String], clients_per_team: u32) -> Self {
        let mut inner = HashMap::with_capacity(teams.len());
        for name in teams {
            inner.insert(name.clone(), (clients_per_team, clients_per_team));
        }
        Self { inner }
    }

    /// Take one slot. On success returns remaining free slots after the join.
    fn try_join(&mut self, team: &str) -> Result<u32, JoinError> {
        let Some((free, _max)) = self.inner.get_mut(team) else {
            return Err(JoinError::Unknown);
        };
        if *free == 0 {
            return Err(JoinError::Full);
        }
        *free -= 1;
        Ok(*free)
    }

    fn release(&mut self, team: &str) {
        if let Some((free, max)) = self.inner.get_mut(team) {
            if *free < *max {
                *free += 1;
            }
        }
    }

    /// Remaining free slots for `team` (for `connect_nbr`).
    fn free(&self, team: &str) -> u32 {
        self.inner.get(team).map(|(free, _)| *free).unwrap_or(0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase {
    /// Waiting for `team-name\n` (after WELCOME is fully sent).
    AwaitingTeam,
    /// Handshake complete; commands arrive in later tickets.
    Joined,
}

struct Connection {
    stream: TcpStream,
    /// Remaining outbound bytes.
    out: Vec<u8>,
    out_pos: usize,
    /// Accumulated inbound bytes until `\n`.
    inbuf: Vec<u8>,
    phase: Phase,
    /// Team joined (for slot release on disconnect).
    team: Option<String>,
    /// Spawned player id after a successful handshake (S05).
    player_id: Option<u32>,
}

impl Connection {
    fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            out: WELCOME.to_vec(),
            out_pos: 0,
            inbuf: Vec::new(),
            phase: Phase::AwaitingTeam,
            team: None,
            player_id: None,
        }
    }

    fn pending_out(&self) -> bool {
        self.out_pos < self.out.len()
    }

    fn queue_out(&mut self, bytes: &[u8]) {
        if !self.pending_out() {
            self.out.clear();
            self.out_pos = 0;
        }
        self.out.extend_from_slice(bytes);
    }

    fn flush_out(&mut self) -> io::Result<()> {
        while self.out_pos < self.out.len() {
            match self.stream.write(&self.out[self.out_pos..]) {
                Ok(0) => {
                    return Err(io::Error::new(
                        ErrorKind::WriteZero,
                        "peer closed during write",
                    ));
                }
                Ok(n) => self.out_pos += n,
                Err(e) if e.kind() == ErrorKind::WouldBlock => return Ok(()),
                Err(e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(e) => return Err(e),
            }
        }
        self.out.clear();
        self.out_pos = 0;
        Ok(())
    }

    /// Read available bytes into `inbuf`. Returns `true` when the peer closed.
    fn read_in(&mut self) -> io::Result<bool> {
        let mut buf = [0u8; 1024];
        loop {
            match self.stream.read(&mut buf) {
                Ok(0) => return Ok(true),
                Ok(n) => {
                    self.inbuf.extend_from_slice(&buf[..n]);
                    // Bound memory if a client never sends `\n`.
                    if self.inbuf.len() > MAX_LINE_BYTES * 4 {
                        return Err(io::Error::new(
                            ErrorKind::InvalidData,
                            "inbound buffer overflow",
                        ));
                    }
                }
                Err(e) if e.kind() == ErrorKind::WouldBlock => return Ok(false),
                Err(e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(e) => return Err(e),
            }
        }
    }

    /// Pop one `\n`-terminated line (strips trailing `\r`). `None` if incomplete.
    fn take_line(&mut self) -> Option<String> {
        let pos = self.inbuf.iter().position(|&b| b == b'\n')?;
        let mut line = self.inbuf.drain(..=pos).collect::<Vec<u8>>();
        line.pop(); // `\n`
        if line.last() == Some(&b'\r') {
            line.pop();
        }
        Some(String::from_utf8_lossy(&line).into_owned())
    }
}

/// Outcome of trying to finish the team handshake for one connection.
enum HandshakeStep {
    /// No action / still waiting for a full line.
    Idle,
    /// Queued `nb-client` + map size; may need WRITABLE interest.
    JoinedNeedsWrite,
    /// Unknown team — caller must print the subject error and drop.
    UnknownTeam(String),
    /// Team exists but has no free slots — drop quietly.
    Full,
}

fn progress_handshake(
    conn: &mut Connection,
    slots: &mut TeamSlots,
    players: &mut PlayerSet,
    world: &World,
    rng: &mut SeededRng,
    width: u32,
    height: u32,
) -> HandshakeStep {
    if conn.pending_out() || conn.phase != Phase::AwaitingTeam {
        return HandshakeStep::Idle;
    }
    let Some(team) = conn.take_line() else {
        return HandshakeStep::Idle;
    };
    match slots.try_join(&team) {
        Ok(remaining) => {
            let id = players.spawn(&team, world, rng);
            conn.team = Some(team);
            conn.player_id = Some(id);
            conn.phase = Phase::Joined;
            let reply = format!("{remaining}\n{width} {height}\n");
            conn.queue_out(reply.as_bytes());
            HandshakeStep::JoinedNeedsWrite
        }
        Err(JoinError::Unknown) => HandshakeStep::UnknownTeam(team),
        Err(JoinError::Full) => HandshakeStep::Full,
    }
}

/// Bind `127.0.0.1:config.port` and serve until a fatal I/O error.
pub fn serve(config: &Config) -> io::Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], config.port));
    serve_addr(addr, config, None)
}

/// Bind `addr` and run the multiplexed event loop.
pub fn serve_addr(
    addr: SocketAddr,
    config: &Config,
    running: Option<&AtomicBool>,
) -> io::Result<()> {
    let listener = TcpListener::bind(addr)?;
    serve_listener(listener, config, running)
}

/// Run the event loop on an already-bound non-blocking [`TcpListener`].
pub fn serve_listener(
    mut listener: TcpListener,
    config: &Config,
    running: Option<&AtomicBool>,
) -> io::Result<()> {
    let mut poll = Poll::new()?;
    let mut events = Events::with_capacity(128);
    poll.registry()
        .register(&mut listener, LISTENER, Interest::READABLE)?;

    let mut connections: HashMap<Token, Connection> = HashMap::new();
    let mut next_token = Token(1);
    let mut slots = TeamSlots::new(&config.teams, config.clients_per_team);
    let width = config.width;
    let height = config.height;

    let world = World::generate_random(width, height);
    eprintln!("server: {}", world.summary_line());
    let mut players = PlayerSet::new();
    let mut rng = SeededRng::new(world.seed ^ 0x0C0F_FEE0_0D15_CAFE);
    let t = config.t;

    if let Ok(addr) = listener.local_addr() {
        eprintln!("server: listening on {addr}");
    }

    loop {
        if running.is_some_and(|r| !r.load(Ordering::SeqCst)) {
            return Ok(());
        }

        let now = Instant::now();
        // Wake sooner when a command delay is about to elapse (AQ06 timings).
        let timeout = match players.next_busy_deadline() {
            Some(deadline) if deadline > now => {
                deadline.saturating_duration_since(now).min(POLL_TIMEOUT)
            }
            Some(_) => Duration::from_millis(1),
            None => POLL_TIMEOUT,
        };

        if let Err(e) = poll.poll(&mut events, Some(timeout)) {
            if e.kind() == ErrorKind::Interrupted {
                continue;
            }
            return Err(e);
        }

        for event in events.iter() {
            match event.token() {
                LISTENER => {
                    accept_pending(&mut poll, &mut listener, &mut connections, &mut next_token)?
                }
                token => handle_connection_event(
                    &mut poll,
                    &mut connections,
                    &mut slots,
                    &mut players,
                    &world,
                    &mut rng,
                    token,
                    event,
                    width,
                    height,
                    t,
                )?,
            }
        }

        tick_command_completions(&mut poll, &mut connections, &mut players, &slots, &world, t);
    }
}

fn accept_pending(
    poll: &mut Poll,
    listener: &mut TcpListener,
    connections: &mut HashMap<Token, Connection>,
    next_token: &mut Token,
) -> io::Result<()> {
    loop {
        match listener.accept() {
            Ok((mut stream, peer)) => {
                let token = *next_token;
                *next_token = Token(next_token.0.wrapping_add(1).max(1));

                poll.registry().register(
                    &mut stream,
                    token,
                    Interest::READABLE.add(Interest::WRITABLE),
                )?;

                let mut conn = Connection::new(stream);
                if let Err(e) = conn.flush_out() {
                    let _ = poll.registry().deregister(&mut conn.stream);
                    eprintln!("server: welcome failed for {peer}: {e}");
                    continue;
                }

                let interest = if conn.pending_out() {
                    Interest::READABLE.add(Interest::WRITABLE)
                } else {
                    Interest::READABLE
                };
                poll.registry()
                    .reregister(&mut conn.stream, token, interest)?;
                connections.insert(token, conn);
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => return Ok(()),
            Err(e) => return Err(e),
        }
    }
}

fn drop_connection(
    poll: &mut Poll,
    connections: &mut HashMap<Token, Connection>,
    slots: &mut TeamSlots,
    players: &mut PlayerSet,
    token: Token,
) {
    if let Some(mut conn) = connections.remove(&token) {
        if let Some(id) = conn.player_id.take() {
            players.remove(id);
        }
        if let Some(team) = conn.team.take() {
            slots.release(&team);
        }
        let _ = poll.registry().deregister(&mut conn.stream);
    }
}

fn set_interest(
    poll: &mut Poll,
    conn: &mut Connection,
    token: Token,
    interest: Interest,
) -> io::Result<()> {
    poll.registry()
        .reregister(&mut conn.stream, token, interest)
}

#[allow(clippy::too_many_arguments)]
fn handle_connection_event(
    poll: &mut Poll,
    connections: &mut HashMap<Token, Connection>,
    slots: &mut TeamSlots,
    players: &mut PlayerSet,
    world: &World,
    rng: &mut SeededRng,
    token: Token,
    event: &mio::event::Event,
    width: u32,
    height: u32,
    t: u32,
) -> io::Result<()> {
    let mut drop_conn = false;
    let mut want_writable = false;

    if let Some(conn) = connections.get_mut(&token) {
        if event.is_writable() && conn.pending_out() {
            match conn.flush_out() {
                Ok(()) => {}
                Err(e) => {
                    eprintln!("server: write error: {e}");
                    drop_conn = true;
                }
            }
        }

        if !drop_conn && event.is_readable() {
            match conn.read_in() {
                Ok(true) => drop_conn = true,
                Ok(false) => {}
                Err(e) => {
                    eprintln!("server: read error: {e}");
                    drop_conn = true;
                }
            }
        }

        if !drop_conn {
            match progress_handshake(conn, slots, players, world, rng, width, height) {
                HandshakeStep::Idle => {}
                HandshakeStep::JoinedNeedsWrite => {
                    want_writable = conn.pending_out();
                    // Keep any pipelined command lines after the team name (S06).
                    if let Some(id) = conn.player_id {
                        if let Some(p) = players.get(id) {
                            eprintln!(
                                "server: player {id} team={} at {},{} level={} life_tu={}",
                                p.team, p.x, p.y, p.level, p.inventory.life_tu
                            );
                        }
                    }
                }
                HandshakeStep::UnknownTeam(name) => {
                    // RQ19 / AQ14 — exact subject wording on the server.
                    eprintln!("Error: the team {name} doesn't exist");
                    drop_conn = true;
                }
                HandshakeStep::Full => {
                    drop_conn = true;
                }
            }
        }

        // After handshake: parse commands into the per-player queue (S06).
        if !drop_conn && conn.phase == Phase::Joined && ingest_player_commands(conn, players, t) {
            want_writable = true;
        }

        if event.is_error() || event.is_read_closed() || event.is_write_closed() {
            drop_conn = true;
        }

        if !drop_conn {
            let interest = if conn.pending_out() || want_writable {
                Interest::READABLE.add(Interest::WRITABLE)
            } else {
                Interest::READABLE
            };
            if let Err(e) = set_interest(poll, conn, token, interest) {
                eprintln!("server: reregister error: {e}");
                drop_conn = true;
            }
        }
    }

    if drop_conn {
        drop_connection(poll, connections, slots, players, token);
    }

    Ok(())
}

/// Read `\n`-terminated lines into the player's command queue.
/// Returns `true` if immediate outbound data was queued (`ko`).
fn ingest_player_commands(conn: &mut Connection, players: &mut PlayerSet, t: u32) -> bool {
    let Some(player_id) = conn.player_id else {
        return false;
    };
    let now = Instant::now();
    let mut queued_ko = false;
    while let Some(line) = conn.take_line() {
        match commands::parse_command(&line) {
            Some(cmd) => {
                if let Some(player) = players.get_mut(player_id) {
                    let _ = player.queue.try_enqueue(cmd, now, t);
                }
            }
            None => {
                conn.queue_out(KO.as_bytes());
                queued_ko = true;
            }
        }
    }
    queued_ko
}

/// Finish delayed commands and queue their protocol replies.
fn tick_command_completions(
    poll: &mut Poll,
    connections: &mut HashMap<Token, Connection>,
    players: &mut PlayerSet,
    slots: &TeamSlots,
    world: &World,
    t: u32,
) {
    let now = Instant::now();
    let tokens: Vec<Token> = connections.keys().copied().collect();
    for token in tokens {
        let Some(conn) = connections.get_mut(&token) else {
            continue;
        };
        let Some(player_id) = conn.player_id else {
            continue;
        };
        let mut wrote = false;
        while let Some(player) = players.get_mut(player_id) {
            let Some(cmd) = player.queue.poll_complete(now, t) else {
                break;
            };
            let reply = complete_command(&cmd, player, slots, world);
            conn.queue_out(reply.as_bytes());
            wrote = true;
        }
        if wrote || conn.pending_out() {
            let _ = set_interest(
                poll,
                conn,
                token,
                Interest::READABLE.add(Interest::WRITABLE),
            );
            let _ = conn.flush_out();
            if !conn.pending_out() {
                let _ = set_interest(poll, conn, token, Interest::READABLE);
            }
        }
    }
}

/// Apply command effects and build the response line(s).
///
/// Movement (`advance` / `left` / `right`) is applied here (S07). Vision,
/// pick/drop, kick, broadcast, ritual, and fork side-effects land later.
fn complete_command(
    cmd: &Command,
    player: &mut crate::player::Player,
    slots: &TeamSlots,
    world: &World,
) -> String {
    match cmd {
        Command::Advance => {
            player.advance(world);
            "ok\n".to_string()
        }
        Command::Right => {
            player.turn_right();
            "ok\n".to_string()
        }
        Command::Left => {
            player.turn_left();
            "ok\n".to_string()
        }
        Command::Fork | Command::Broadcast(_) | Command::Kick => "ok\n".to_string(),
        Command::See => "{}\n".to_string(),
        Command::Inventory => player.inventory_reply(),
        Command::Pick(_) | Command::Drop(_) => "ko\n".to_string(),
        Command::Enchantment => "evolution in progress\n".to_string(),
        Command::ConnectNbr => format!("{}\n", slots.free(&player.team)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    fn test_config(clients_per_team: u32) -> Config {
        Config {
            port: 1,
            width: 10,
            height: 20,
            teams: vec!["alpha".into(), "beta".into()],
            clients_per_team,
            t: 100,
        }
    }

    fn read_line(stream: &mut std::net::TcpStream) -> io::Result<Vec<u8>> {
        stream.set_read_timeout(Some(Duration::from_secs(2)))?;
        let mut buf = Vec::new();
        let mut byte = [0u8; 1];
        loop {
            stream.read_exact(&mut byte)?;
            buf.push(byte[0]);
            if byte[0] == b'\n' {
                return Ok(buf);
            }
            if buf.len() > 256 {
                return Err(io::Error::new(ErrorKind::InvalidData, "line too long"));
            }
        }
    }

    fn spawn_server(
        config: Config,
    ) -> (
        SocketAddr,
        Arc<AtomicBool>,
        thread::JoinHandle<io::Result<()>>,
    ) {
        let std_listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
        std_listener.set_nonblocking(true).expect("nonblocking");
        let addr = std_listener.local_addr().expect("local_addr");
        let listener = TcpListener::from_std(std_listener);

        let running = Arc::new(AtomicBool::new(true));
        let flag = Arc::clone(&running);
        let handle = thread::spawn(move || serve_listener(listener, &config, Some(&flag)));
        thread::sleep(Duration::from_millis(20));
        (addr, running, handle)
    }

    fn stop_server(running: Arc<AtomicBool>, handle: thread::JoinHandle<io::Result<()>>) {
        running.store(false, Ordering::SeqCst);
        handle
            .join()
            .expect("server thread panicked")
            .expect("server I/O error");
    }

    fn handshake(addr: SocketAddr, team: &str) -> io::Result<(std::net::TcpStream, u32, u32, u32)> {
        let mut client = std::net::TcpStream::connect(addr)?;
        client.set_read_timeout(Some(Duration::from_secs(2)))?;
        client.set_write_timeout(Some(Duration::from_secs(2)))?;

        let welcome = read_line(&mut client)?;
        assert_eq!(welcome, WELCOME);

        client.write_all(format!("{team}\n").as_bytes())?;

        let nb_line = read_line(&mut client)?;
        let nb: u32 = std::str::from_utf8(&nb_line[..nb_line.len() - 1])
            .unwrap()
            .parse()
            .expect("nb-client");

        let dim_line = read_line(&mut client)?;
        let dim = std::str::from_utf8(&dim_line[..dim_line.len() - 1]).unwrap();
        let mut parts = dim.split_whitespace();
        let w: u32 = parts.next().unwrap().parse().unwrap();
        let h: u32 = parts.next().unwrap().parse().unwrap();
        assert!(parts.next().is_none());

        Ok((client, nb, w, h))
    }

    #[test]
    fn welcome_bytes_match_subject() {
        assert_eq!(WELCOME, b"WELCOME\n");
    }

    #[test]
    fn client_receives_welcome_on_connect() {
        let (addr, running, handle) = spawn_server(test_config(5));
        let mut client = std::net::TcpStream::connect(addr).expect("connect");
        let line = read_line(&mut client).expect("read WELCOME");
        assert_eq!(line, WELCOME);
        stop_server(running, handle);
    }

    #[test]
    fn multiple_clients_each_receive_welcome() {
        let (addr, running, handle) = spawn_server(test_config(5));
        for _ in 0..3 {
            let mut client = std::net::TcpStream::connect(addr).expect("connect");
            let line = read_line(&mut client).expect("read WELCOME");
            assert_eq!(line, WELCOME);
        }
        stop_server(running, handle);
    }

    #[test]
    fn valid_team_receives_remaining_slots_and_map_size() {
        let (addr, running, handle) = spawn_server(test_config(5));
        let (_client, nb, w, h) = handshake(addr, "alpha").expect("handshake");
        assert_eq!(nb, 4); // 5 - 1
        assert_eq!((w, h), (10, 20));
        stop_server(running, handle);
    }

    #[test]
    fn nb_client_decrements_per_join() {
        let (addr, running, handle) = spawn_server(test_config(3));
        let (_c1, nb1, _, _) = handshake(addr, "alpha").expect("first");
        let (_c2, nb2, _, _) = handshake(addr, "alpha").expect("second");
        assert_eq!(nb1, 2);
        assert_eq!(nb2, 1);
        stop_server(running, handle);
    }

    #[test]
    fn unknown_team_is_disconnected_without_handshake_reply() {
        let (addr, running, handle) = spawn_server(test_config(5));
        let mut client = std::net::TcpStream::connect(addr).expect("connect");
        assert_eq!(read_line(&mut client).unwrap(), WELCOME);
        client.write_all(b"wrong_team\n").unwrap();

        // Peer should close; a subsequent read yields 0 or an error, not nb-client.
        client
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let mut buf = [0u8; 32];
        match client.read(&mut buf) {
            Ok(0) => {}
            Err(e)
                if e.kind() == ErrorKind::ConnectionReset
                    || e.kind() == ErrorKind::WouldBlock
                    || e.kind() == ErrorKind::TimedOut => {}
            other => panic!("expected disconnect, got {other:?}"),
        }

        stop_server(running, handle);
    }

    #[test]
    fn full_team_rejects_additional_client() {
        let (addr, running, handle) = spawn_server(test_config(1));
        let (keep, nb, _, _) = handshake(addr, "beta").expect("only slot");
        assert_eq!(nb, 0);

        let mut client = std::net::TcpStream::connect(addr).expect("connect");
        assert_eq!(read_line(&mut client).unwrap(), WELCOME);
        client.write_all(b"beta\n").unwrap();
        client
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let mut buf = [0u8; 32];
        match client.read(&mut buf) {
            Ok(0) => {}
            Err(e)
                if e.kind() == ErrorKind::ConnectionReset
                    || e.kind() == ErrorKind::WouldBlock
                    || e.kind() == ErrorKind::TimedOut => {}
            other => panic!("expected disconnect when full, got {other:?}"),
        }

        drop(keep);
        stop_server(running, handle);
    }

    #[test]
    fn team_slots_try_join_and_release() {
        let mut slots = TeamSlots::new(&[String::from("a")], 2);
        assert_eq!(slots.try_join("a"), Ok(1));
        assert_eq!(slots.try_join("a"), Ok(0));
        assert_eq!(slots.try_join("a"), Err(JoinError::Full));
        assert_eq!(slots.try_join("nope"), Err(JoinError::Unknown));
        slots.release("a");
        assert_eq!(slots.try_join("a"), Ok(0));
    }

    #[test]
    fn telnet_style_crlf_team_line_accepted() {
        let (addr, running, handle) = spawn_server(test_config(2));
        let mut client = std::net::TcpStream::connect(addr).expect("connect");
        assert_eq!(read_line(&mut client).unwrap(), WELCOME);
        client.write_all(b"alpha\r\n").unwrap();
        let nb_line = read_line(&mut client).unwrap();
        assert_eq!(nb_line, b"1\n");
        let dim = read_line(&mut client).unwrap();
        assert_eq!(dim, b"10 20\n");
        stop_server(running, handle);
    }

    #[test]
    fn unknown_command_gets_immediate_ko() {
        let mut cfg = test_config(5);
        cfg.t = 1000;
        let (addr, running, handle) = spawn_server(cfg);
        let (mut client, _, _, _) = handshake(addr, "alpha").expect("handshake");
        client.write_all(b"not_a_command\n").unwrap();
        let line = read_line(&mut client).expect("ko");
        assert_eq!(line, b"ko\n");
        stop_server(running, handle);
    }

    #[test]
    fn connect_nbr_and_inventory_honor_delays() {
        let mut cfg = test_config(5);
        cfg.t = 1000; // 1/t = 1ms for inventory
        let (addr, running, handle) = spawn_server(cfg);
        let (mut client, _, _, _) = handshake(addr, "alpha").expect("handshake");

        client.write_all(b"connect_nbr\n").unwrap();
        let nbr = read_line(&mut client).expect("connect_nbr");
        assert_eq!(nbr, b"4\n"); // 5 - 1 joined

        client.write_all(b"inventory\n").unwrap();
        let inv = read_line(&mut client).expect("inventory");
        let inv_str = std::str::from_utf8(&inv).unwrap();
        assert!(inv_str.starts_with("{food 1260,"));
        assert!(inv_str.ends_with("}\n"));

        stop_server(running, handle);
    }

    #[test]
    fn advance_left_right_reply_ok() {
        let mut cfg = test_config(5);
        cfg.t = 1000;
        let (addr, running, handle) = spawn_server(cfg);
        let (mut client, _, _, _) = handshake(addr, "alpha").expect("handshake");
        for cmd in ["right\n", "advance\n", "left\n"] {
            client.write_all(cmd.as_bytes()).unwrap();
            assert_eq!(read_line(&mut client).unwrap(), b"ok\n");
        }
        stop_server(running, handle);
    }
}
