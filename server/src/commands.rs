//! Command parse, delays, and per-player request queue (S06).
//!
//! - Syntax and delays match the subject table (RQ11).
//! - Unknown / incorrect lines → caller sends `ko` (not queued).
//! - At most [`MAX_PENDING`] successful requests may await a response (RQ12);
//!   further valid commands are ignored until a slot frees.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use crate::time::action_duration;

/// Max in-flight commands per player (executing + waiting).
pub const MAX_PENDING: usize = 10;

/// Immediate rejection reply for unknown / malformed input.
pub const KO: &str = "ko\n";

/// Parsed player command (lowercase subject syntax).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Advance,
    Right,
    Left,
    See,
    Inventory,
    Pick(String),
    Drop(String),
    Kick,
    Broadcast(String),
    Enchantment,
    Fork,
    ConnectNbr,
}

impl Command {
    /// Delay cost in time units (numerator of `cost/t`).
    pub fn delay_tu(&self) -> u32 {
        match self {
            Command::Advance
            | Command::Right
            | Command::Left
            | Command::See
            | Command::Pick(_)
            | Command::Drop(_)
            | Command::Kick
            | Command::Broadcast(_) => 7,
            Command::Inventory => 1,
            Command::Enchantment => 300,
            Command::Fork => 48,
            Command::ConnectNbr => 0,
        }
    }

    pub fn delay(&self, t: u32) -> Duration {
        action_duration(t, self.delay_tu())
    }
}

/// Parse one protocol line (without trailing `\n`). `None` → respond with [`KO`].
pub fn parse_command(line: &str) -> Option<Command> {
    let line = line.trim_end_matches('\r');
    if line.is_empty() {
        return None;
    }

    // `broadcast <text>` — text may contain spaces; may be empty after the verb.
    if let Some(rest) = line.strip_prefix("broadcast") {
        if rest.is_empty() {
            return Some(Command::Broadcast(String::new()));
        }
        if let Some(text) = rest.strip_prefix(' ') {
            return Some(Command::Broadcast(text.to_string()));
        }
        return None;
    }

    let mut parts = line.split_whitespace();
    let verb = parts.next()?;
    match verb {
        "advance" if parts.next().is_none() => Some(Command::Advance),
        "right" if parts.next().is_none() => Some(Command::Right),
        "left" if parts.next().is_none() => Some(Command::Left),
        "see" if parts.next().is_none() => Some(Command::See),
        "inventory" if parts.next().is_none() => Some(Command::Inventory),
        "kick" if parts.next().is_none() => Some(Command::Kick),
        "enchantment" if parts.next().is_none() => Some(Command::Enchantment),
        "fork" if parts.next().is_none() => Some(Command::Fork),
        "connect_nbr" if parts.next().is_none() => Some(Command::ConnectNbr),
        "pick" => {
            let obj = parts.next()?;
            if parts.next().is_some() {
                return None;
            }
            Some(Command::Pick(obj.to_string()))
        }
        "drop" => {
            let obj = parts.next()?;
            if parts.next().is_some() {
                return None;
            }
            Some(Command::Drop(obj.to_string()))
        }
        _ => None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Queued {
    cmd: Command,
}

/// Per-player FIFO of successful requests awaiting a response.
#[derive(Debug, Default)]
pub struct CmdQueue {
    /// Active command currently waiting out its delay (`None` if idle).
    active: Option<Queued>,
    /// When `active` completes.
    busy_until: Option<Instant>,
    /// Commands waiting to become active (not yet started).
    waiting: VecDeque<Queued>,
}

impl CmdQueue {
    pub fn new() -> Self {
        Self::default()
    }

    /// Total successful requests without a response yet.
    pub fn pending_count(&self) -> usize {
        self.waiting.len() + usize::from(self.active.is_some())
    }

    pub fn is_busy(&self) -> bool {
        self.active.is_some()
    }

    pub fn busy_until(&self) -> Option<Instant> {
        self.busy_until
    }

    /// Enqueue a valid command. Returns `false` if the buffer is full (ignored).
    pub fn try_enqueue(&mut self, cmd: Command, now: Instant, t: u32) -> bool {
        if self.pending_count() >= MAX_PENDING {
            return false;
        }
        if self.active.is_none() {
            self.start(cmd, now, t);
        } else {
            self.waiting.push_back(Queued { cmd });
        }
        true
    }

    fn start(&mut self, cmd: Command, now: Instant, t: u32) {
        let delay = cmd.delay(t);
        self.busy_until = Some(now + delay);
        self.active = Some(Queued { cmd });
    }

    /// If the active command's delay has elapsed, pop it and start the next.
    pub fn poll_complete(&mut self, now: Instant, t: u32) -> Option<Command> {
        let until = self.busy_until?;
        if now < until {
            return None;
        }
        let finished = self.active.take()?.cmd;
        self.busy_until = None;
        if let Some(next) = self.waiting.pop_front() {
            self.start(next.cmd, now, t);
        }
        Some(finished)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_subject_verbs() {
        assert_eq!(parse_command("advance"), Some(Command::Advance));
        assert_eq!(parse_command("right"), Some(Command::Right));
        assert_eq!(parse_command("left"), Some(Command::Left));
        assert_eq!(parse_command("see"), Some(Command::See));
        assert_eq!(parse_command("inventory"), Some(Command::Inventory));
        assert_eq!(parse_command("kick"), Some(Command::Kick));
        assert_eq!(parse_command("enchantment"), Some(Command::Enchantment));
        assert_eq!(parse_command("fork"), Some(Command::Fork));
        assert_eq!(parse_command("connect_nbr"), Some(Command::ConnectNbr));
        assert_eq!(
            parse_command("pick jade"),
            Some(Command::Pick("jade".into()))
        );
        assert_eq!(
            parse_command("drop food"),
            Some(Command::Drop("food".into()))
        );
        assert_eq!(
            parse_command("broadcast hello world"),
            Some(Command::Broadcast("hello world".into()))
        );
        assert_eq!(
            parse_command("broadcast"),
            Some(Command::Broadcast(String::new()))
        );
    }

    #[test]
    fn unknown_or_malformed_are_errors() {
        assert!(parse_command("Advance").is_none()); // case-sensitive
        assert!(parse_command("noop").is_none());
        assert!(parse_command("pick").is_none());
        assert!(parse_command("pick a b").is_none());
        assert!(parse_command("advance now").is_none());
        assert!(parse_command("").is_none());
    }

    #[test]
    fn delays_match_subject_table() {
        assert_eq!(Command::Advance.delay_tu(), 7);
        assert_eq!(Command::Inventory.delay_tu(), 1);
        assert_eq!(Command::Enchantment.delay_tu(), 300);
        assert_eq!(Command::Fork.delay_tu(), 48);
        assert_eq!(Command::ConnectNbr.delay_tu(), 0);
    }

    #[test]
    fn queue_caps_at_ten() {
        let mut q = CmdQueue::new();
        let now = Instant::now();
        for _ in 0..MAX_PENDING {
            assert!(q.try_enqueue(Command::Advance, now, 100));
        }
        assert!(!q.try_enqueue(Command::Advance, now, 100));
        assert_eq!(q.pending_count(), MAX_PENDING);
    }

    #[test]
    fn queue_completes_in_order_after_delays() {
        let mut q = CmdQueue::new();
        let t = 1000; // fast: 7/t = 7ms
        let start = Instant::now();
        assert!(q.try_enqueue(Command::ConnectNbr, start, t)); // 0 delay
        assert!(q.try_enqueue(Command::Inventory, start, t)); // 1/t

        // ConnectNbr finishes immediately.
        let done = q.poll_complete(start, t).expect("connect_nbr");
        assert_eq!(done, Command::ConnectNbr);

        // Inventory still running.
        assert!(q.poll_complete(start, t).is_none());
        let later = start + action_duration(t, 1);
        let done = q.poll_complete(later, t).expect("inventory");
        assert_eq!(done, Command::Inventory);
        assert_eq!(q.pending_count(), 0);
    }
}
