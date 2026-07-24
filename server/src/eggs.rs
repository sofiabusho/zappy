//! Fork / ship / eggs (S13 / RQ13 / AQ26).
//!
//! `fork` (48/t) calls a ship at the player's tile. After **600** time units the
//! ship arrives: the team gains one free connection slot (`connect_nbr` rises)
//! and the next joining teammate spawns on that tile with a random facing.

use std::collections::VecDeque;

/// Subject ship travel time after `fork` completes (RQ13).
pub const SHIP_ARRIVAL_TU: u64 = 600;

/// An incubating or hatched ship/egg for one team.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Egg {
    pub team: String,
    pub x: u32,
    pub y: u32,
    /// Game time unit when the ship arrives / egg hatches.
    pub hatch_at_tu: u64,
}

/// Pending ships: incubating until `hatch_at_tu`, then ready for a join spawn.
#[derive(Debug, Default)]
pub struct EggSet {
    incubating: Vec<Egg>,
    /// Hatched eggs waiting for a client to take the new slot (FIFO per take).
    ready: VecDeque<Egg>,
}

impl EggSet {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a ship called by `fork` at `(x, y)` for `team`.
    pub fn call_ship(&mut self, team: impl Into<String>, x: u32, y: u32, now_tu: u64) {
        self.incubating.push(Egg {
            team: team.into(),
            x,
            y,
            hatch_at_tu: now_tu.saturating_add(SHIP_ARRIVAL_TU),
        });
    }

    /// Hatch every egg whose ship has arrived. Returns how many hatched.
    ///
    /// Caller must increase team slots for each returned egg's team.
    pub fn hatch_due(&mut self, now_tu: u64) -> Vec<Egg> {
        let (due, rest): (Vec<_>, Vec<_>) = self
            .incubating
            .drain(..)
            .partition(|e| e.hatch_at_tu <= now_tu);
        self.incubating = rest;
        for egg in &due {
            self.ready.push_back(egg.clone());
        }
        due
    }

    /// Take the oldest ready egg for `team`, if any (spawn site for a new join).
    pub fn take_spawn(&mut self, team: &str) -> Option<(u32, u32)> {
        let idx = self.ready.iter().position(|e| e.team == team)?;
        let egg = self.ready.remove(idx).unwrap();
        Some((egg.x, egg.y))
    }

    pub fn incubating_len(&self) -> usize {
        self.incubating.len()
    }

    pub fn ready_len(&self) -> usize {
        self.ready.len()
    }

    /// Earliest hatch deadline among incubating eggs (for poll wakeups).
    pub fn next_hatch_tu(&self) -> Option<u64> {
        self.incubating.iter().map(|e| e.hatch_at_tu).min()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ship_arrives_after_600_tu() {
        let mut eggs = EggSet::new();
        eggs.call_ship("alpha", 2, 3, 10);
        assert_eq!(eggs.hatch_due(609).len(), 0);
        let hatched = eggs.hatch_due(610);
        assert_eq!(hatched.len(), 1);
        assert_eq!(hatched[0].team, "alpha");
        assert_eq!((hatched[0].x, hatched[0].y), (2, 3));
        assert_eq!(eggs.take_spawn("alpha"), Some((2, 3)));
        assert_eq!(eggs.take_spawn("alpha"), None);
    }

    #[test]
    fn take_spawn_is_team_fifo() {
        let mut eggs = EggSet::new();
        eggs.call_ship("a", 1, 1, 0);
        eggs.call_ship("b", 5, 5, 0);
        eggs.call_ship("a", 2, 2, 0);
        let _ = eggs.hatch_due(600);
        assert_eq!(eggs.take_spawn("a"), Some((1, 1)));
        assert_eq!(eggs.take_spawn("a"), Some((2, 2)));
        assert_eq!(eggs.take_spawn("b"), Some((5, 5)));
    }

    #[test]
    fn ship_arrival_constant_matches_subject() {
        assert_eq!(SHIP_ARRIVAL_TU, 600);
    }
}
