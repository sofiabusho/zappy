//! Zappy game server library.
//!
//! Modules land per ticket. `cli` (S01) parses the server command line;
//! `net` (S02/S03) owns the multiplexed TCP accept loop, `WELCOME`, and team
//! handshake; `world` (S04) owns the toroidal map and resource generation;
//! `player` (S05) owns spawn loadout and team membership; `time` / `commands`
//! (S06) own the scheduler and per-player request queue; `vision` (S08) formats
//! `see` replies; `kick` (S11) pushes co-tile players with `moving <K>`;
//! `broadcast` (S12) delivers directional `message <K>,<text>` sound;
//! `eggs` (S13) tracks fork ships and hatch slots;
//! `ritual` (S14) runs enchantment elevation;
//! `win` (S15) detects six teammates at level 8.

pub mod broadcast;
pub mod cli;
pub mod commands;
pub mod eggs;
pub mod kick;
pub mod net;
pub mod player;
pub mod ritual;
pub mod time;
pub mod vision;
pub mod win;
pub mod world;
