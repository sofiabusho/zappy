//! Zappy game server library.
//!
//! Modules land per ticket. `cli` (S01) parses the server command line;
//! `net` (S02/S03) owns the multiplexed TCP accept loop, `WELCOME`, and team
//! handshake; `world` (S04) owns the toroidal map and resource generation.

pub mod cli;
pub mod net;
pub mod world;
