//! Zappy game server library.
//!
//! Modules land per ticket. `cli` (S01) parses the server command line;
//! `net` (S02/S03) owns the multiplexed TCP accept loop, `WELCOME`, and team
//! handshake. World and game logic arrive in later server tickets.

pub mod cli;
pub mod net;
