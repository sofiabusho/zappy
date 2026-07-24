//! Multiplexed TCP accept loop (S02).
//!
//! Binds a non-blocking listener, accepts clients via `mio` poll, and sends
//! [`WELCOME`] immediately on connect (RQ16 / AQ02). Full handshake (team /
//! nb-client / map size) arrives in S03; inbound bytes are drained and ignored
//! here so a pipelining client cannot stall the event loop.

use std::collections::HashMap;
use std::io::{self, ErrorKind, Read, Write};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Token};

use crate::cli::Config;

/// Exact welcome line from the subject handshake table (`WELCOME\n`).
pub const WELCOME: &[u8] = b"WELCOME\n";

const LISTENER: Token = Token(0);
/// Poll wake interval so the loop never blocks forever (RQ16) and tests can stop.
const POLL_TIMEOUT: Duration = Duration::from_millis(50);

struct Connection {
    stream: TcpStream,
    /// Remaining outbound bytes (starts as [`WELCOME`]).
    out: Vec<u8>,
    out_pos: usize,
}

impl Connection {
    fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            out: WELCOME.to_vec(),
            out_pos: 0,
        }
    }

    fn pending_out(&self) -> bool {
        self.out_pos < self.out.len()
    }

    /// Write as much of the pending buffer as the socket will accept.
    fn flush_out(&mut self) -> io::Result<()> {
        while self.out_pos < self.out.len() {
            match self.stream.write(&self.out[self.out_pos..]) {
                Ok(0) => {
                    return Err(io::Error::new(
                        ErrorKind::WriteZero,
                        "peer closed during welcome",
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

    /// Drain readable data. Returns `true` when the peer has closed.
    fn drain_in(&mut self) -> io::Result<bool> {
        let mut buf = [0u8; 1024];
        loop {
            match self.stream.read(&mut buf) {
                Ok(0) => return Ok(true),
                Ok(_) => {
                    // Handshake / commands are parsed in S03+; discard for now.
                }
                Err(e) if e.kind() == ErrorKind::WouldBlock => return Ok(false),
                Err(e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(e) => return Err(e),
            }
        }
    }
}

/// Bind `127.0.0.1:config.port` and serve until a fatal I/O error.
pub fn serve(config: &Config) -> io::Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], config.port));
    serve_addr(addr, None)
}

/// Bind `addr` and run the multiplexed event loop.
///
/// When `running` is `Some`, the loop exits cleanly once the flag is false
/// (used by tests). When `None`, runs until a fatal I/O error.
pub fn serve_addr(addr: SocketAddr, running: Option<&AtomicBool>) -> io::Result<()> {
    let listener = TcpListener::bind(addr)?;
    serve_listener(listener, running)
}

/// Run the event loop on an already-bound non-blocking [`TcpListener`].
pub fn serve_listener(mut listener: TcpListener, running: Option<&AtomicBool>) -> io::Result<()> {
    let mut poll = Poll::new()?;
    let mut events = Events::with_capacity(128);
    poll.registry()
        .register(&mut listener, LISTENER, Interest::READABLE)?;

    let mut connections: HashMap<Token, Connection> = HashMap::new();
    let mut next_token = Token(1);

    if let Ok(addr) = listener.local_addr() {
        eprintln!("server: listening on {addr}");
    }

    loop {
        if running.is_some_and(|r| !r.load(Ordering::SeqCst)) {
            return Ok(());
        }

        if let Err(e) = poll.poll(&mut events, Some(POLL_TIMEOUT)) {
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
                token => {
                    handle_connection_event(&mut poll, &mut connections, token, event)?;
                }
            }
        }
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

                if !conn.pending_out() {
                    poll.registry()
                        .reregister(&mut conn.stream, token, Interest::READABLE)?;
                }
                connections.insert(token, conn);
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => return Ok(()),
            Err(e) => return Err(e),
        }
    }
}

fn handle_connection_event(
    poll: &mut Poll,
    connections: &mut HashMap<Token, Connection>,
    token: Token,
    event: &mio::event::Event,
) -> io::Result<()> {
    let mut drop_conn = false;
    let mut reregister_readable = false;

    if let Some(conn) = connections.get_mut(&token) {
        if event.is_writable() && conn.pending_out() {
            match conn.flush_out() {
                Ok(()) if !conn.pending_out() => reregister_readable = true,
                Ok(()) => {}
                Err(e) => {
                    eprintln!("server: write error: {e}");
                    drop_conn = true;
                }
            }
        }

        if !drop_conn && event.is_readable() {
            match conn.drain_in() {
                Ok(closed) => drop_conn = closed,
                Err(e) => {
                    eprintln!("server: read error: {e}");
                    drop_conn = true;
                }
            }
        }

        if event.is_error() || event.is_read_closed() || event.is_write_closed() {
            drop_conn = true;
        }

        if reregister_readable && !drop_conn {
            poll.registry()
                .reregister(&mut conn.stream, token, Interest::READABLE)?;
        }
    }

    if drop_conn {
        if let Some(mut conn) = connections.remove(&token) {
            let _ = poll.registry().deregister(&mut conn.stream);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

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
            if buf.len() > 64 {
                return Err(io::Error::new(ErrorKind::InvalidData, "line too long"));
            }
        }
    }

    fn spawn_server() -> (
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
        let handle = thread::spawn(move || serve_listener(listener, Some(&flag)));
        // Brief pause so the poll thread is registered before clients connect.
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

    #[test]
    fn welcome_bytes_match_subject() {
        assert_eq!(WELCOME, b"WELCOME\n");
        assert_eq!(std::str::from_utf8(WELCOME).unwrap(), "WELCOME\n");
    }

    #[test]
    fn client_receives_welcome_on_connect() {
        let (addr, running, handle) = spawn_server();

        let mut client = std::net::TcpStream::connect(addr).expect("connect");
        let line = read_line(&mut client).expect("read WELCOME");
        assert_eq!(line, WELCOME);

        stop_server(running, handle);
    }

    #[test]
    fn multiple_clients_each_receive_welcome() {
        let (addr, running, handle) = spawn_server();

        for _ in 0..3 {
            let mut client = std::net::TcpStream::connect(addr).expect("connect");
            let line = read_line(&mut client).expect("read WELCOME");
            assert_eq!(line, WELCOME);
        }

        stop_server(running, handle);
    }
}
