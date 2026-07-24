//! Zappy game server library.
//!
//! Modules land per ticket. `cli` (S01) parses the server command line;
//! `net` (S02/S03) owns the multiplexed TCP accept loop, `WELCOME`, and team
//! handshake; `world` (S04) owns the toroidal map and resource generation;
//! `player` (S05) owns spawn loadout and team membership; `time` / `commands`
//! (S06) own the scheduler and per-player request queue.

pub mod cli;
pub mod commands;
pub mod net;
pub mod player;
pub mod time;
pub mod world;
