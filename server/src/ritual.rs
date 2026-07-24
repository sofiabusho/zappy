//! Evolution ritual / `enchantment` (S14 / RQ09 / AQ25 / AQ31).
//!
//! Requirements match the subject table exactly. Stones are gathered on the
//! square; under our tile rule (at most one of each type), the ritual **pool**
//! is tile presence plus stones in eligible players' inventories on that tile
//! (documented in `docs/SDS.md`). Player counts follow the **table** (level
//! 1→2 needs one player), not the prose “two or more” line.
//!
//! Flow: starter's `enchantment` begins immediately → all participants get
//! `evolution in progress\n` and `in_ritual`; after 300/t, success yields
//! `current level : K\n` (stones consumed, levels +1). If a multi-player ritual
//! is left with only one living participant mid-flight, it is cancelled and
//! completion replies `ko`.

use crate::player::PlayerSet;
use crate::world::{StoneKind, World};

/// One row of the subject ritual table (from level `from_level` → `from_level+1`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RitualReq {
    pub from_level: u8,
    pub players: u8,
    /// Counts for jade, peridot, amber, amethyst, garnet, ammolite.
    pub stones: [u8; 6],
}

/// Subject table (RQ09 / AQ31).
pub const RITUAL_TABLE: [RitualReq; 7] = [
    RitualReq {
        from_level: 1,
        players: 1,
        stones: [1, 0, 0, 0, 0, 0],
    },
    RitualReq {
        from_level: 2,
        players: 2,
        stones: [1, 1, 1, 0, 0, 0],
    },
    RitualReq {
        from_level: 3,
        players: 2,
        stones: [2, 0, 1, 0, 2, 0],
    },
    RitualReq {
        from_level: 4,
        players: 4,
        stones: [1, 1, 2, 0, 1, 0],
    },
    RitualReq {
        from_level: 5,
        players: 4,
        stones: [1, 2, 1, 3, 0, 0],
    },
    RitualReq {
        from_level: 6,
        players: 6,
        stones: [1, 2, 3, 0, 1, 0],
    },
    RitualReq {
        from_level: 7,
        players: 6,
        stones: [2, 2, 2, 2, 2, 1],
    },
];

/// Look up requirements for elevating from `level` (1..=7).
pub fn requirements_for(level: u8) -> Option<&'static RitualReq> {
    RITUAL_TABLE.iter().find(|r| r.from_level == level)
}

/// In-flight enchantment started by one player.
#[derive(Debug, Clone)]
pub struct ActiveRitual {
    pub starter_id: u32,
    pub level: u8,
    pub x: u32,
    pub y: u32,
    pub participants: Vec<u32>,
}

/// Registry of rituals in progress.
#[derive(Debug, Default)]
pub struct RitualSet {
    active: Vec<ActiveRitual>,
}

impl RitualSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_by_starter(&self, starter_id: u32) -> Option<&ActiveRitual> {
        self.active.iter().find(|r| r.starter_id == starter_id)
    }

    /// Cancel an in-flight ritual started by `starter_id`, if any.
    pub fn cancel_starter(&mut self, starter_id: u32, players: &mut PlayerSet) {
        if let Some(idx) = self.active.iter().position(|r| r.starter_id == starter_id) {
            let ritual = self.active.remove(idx);
            clear_ritual_flags(players, &ritual.participants);
        }
    }

    /// Start a ritual for `starter_id` if eligible. Marks participants `in_ritual`.
    ///
    /// Returns participant ids (including starter) on success.
    #[allow(clippy::result_unit_err)]
    pub fn try_begin(
        &mut self,
        starter_id: u32,
        players: &mut PlayerSet,
        world: &World,
    ) -> Result<Vec<u32>, ()> {
        if self.active.iter().any(|r| r.starter_id == starter_id) {
            return Err(());
        }
        let Some(starter) = players.get(starter_id) else {
            return Err(());
        };
        if starter.in_ritual {
            return Err(());
        }
        let level = starter.level;
        let Some(req) = requirements_for(level) else {
            return Err(());
        };
        let x = starter.x;
        let y = starter.y;
        let participants = eligible_on_tile(players, x, y, level);
        if participants.len() < usize::from(req.players) {
            return Err(());
        }
        if !stones_sufficient(req, world, players, &participants, x, y) {
            return Err(());
        }
        // Cap at required count: first N eligible (starter first if present).
        let mut chosen = Vec::with_capacity(usize::from(req.players));
        if participants.contains(&starter_id) {
            chosen.push(starter_id);
        }
        for id in &participants {
            if chosen.len() >= usize::from(req.players) {
                break;
            }
            if !chosen.contains(id) {
                chosen.push(*id);
            }
        }
        if chosen.len() < usize::from(req.players) {
            return Err(());
        }
        for &id in &chosen {
            if let Some(p) = players.get_mut(id) {
                p.in_ritual = true;
            }
        }
        self.active.push(ActiveRitual {
            starter_id,
            level,
            x,
            y,
            participants: chosen.clone(),
        });
        Ok(chosen)
    }

    /// Finish the starter's ritual after 300/t. On success levels participants
    /// and returns `(new_level, participant_ids_still_present)`.
    #[allow(clippy::result_unit_err)]
    pub fn try_complete(
        &mut self,
        starter_id: u32,
        players: &mut PlayerSet,
        world: &mut World,
    ) -> Result<(u8, Vec<u32>), ()> {
        let idx = self
            .active
            .iter()
            .position(|r| r.starter_id == starter_id)
            .ok_or(())?;
        let ritual = self.active.remove(idx);
        let Some(req) = requirements_for(ritual.level) else {
            clear_ritual_flags(players, &ritual.participants);
            return Err(());
        };

        // Participants still alive, same level, still on the ritual tile.
        let still: Vec<u32> = ritual
            .participants
            .iter()
            .copied()
            .filter(|&id| {
                players
                    .get(id)
                    .is_some_and(|p| p.level == ritual.level && p.x == ritual.x && p.y == ritual.y)
            })
            .collect();

        if still.len() < usize::from(req.players) {
            clear_ritual_flags(players, &ritual.participants);
            return Err(());
        }
        if !stones_sufficient(req, world, players, &still, ritual.x, ritual.y) {
            clear_ritual_flags(players, &ritual.participants);
            return Err(());
        }

        consume_stones(req, world, players, &still, ritual.x, ritual.y);
        let new_level = ritual.level.saturating_add(1);
        for &id in &still {
            if let Some(p) = players.get_mut(id) {
                p.level = new_level;
                p.in_ritual = false;
            }
        }
        // Clear flags on anyone who left mid-ritual.
        clear_ritual_flags(players, &ritual.participants);
        Ok((new_level, still))
    }

    /// After a player dies or disconnects: drop them from rituals; cancel if a
    /// multi-player ritual is left with fewer than required participants.
    pub fn on_player_gone(&mut self, player_id: u32, players: &mut PlayerSet) {
        let mut cancel: Vec<usize> = Vec::new();
        for (i, ritual) in self.active.iter_mut().enumerate() {
            ritual.participants.retain(|&id| id != player_id);
            let Some(req) = requirements_for(ritual.level) else {
                cancel.push(i);
                continue;
            };
            if ritual.participants.len() < usize::from(req.players) {
                cancel.push(i);
            }
        }
        for i in cancel.into_iter().rev() {
            let ritual = self.active.remove(i);
            clear_ritual_flags(players, &ritual.participants);
        }
    }
}

fn clear_ritual_flags(players: &mut PlayerSet, ids: &[u32]) {
    for &id in ids {
        if let Some(p) = players.get_mut(id) {
            p.in_ritual = false;
        }
    }
}

/// Players on `(x,y)` at `level` (any team).
pub fn eligible_on_tile(players: &PlayerSet, x: u32, y: u32, level: u8) -> Vec<u32> {
    players
        .iter()
        .filter(|p| p.x == x && p.y == y && p.level == level && !p.in_ritual)
        .map(|p| p.id)
        .collect()
}

fn stone_count_in_pool(
    kind: StoneKind,
    world: &World,
    players: &PlayerSet,
    participants: &[u32],
    x: u32,
    y: u32,
) -> u32 {
    let mut n = u32::from(world.tile(x, y).has_stone(kind));
    for &id in participants {
        if let Some(p) = players.get(id) {
            n += p.inventory.stone(kind);
        }
    }
    n
}

fn stones_sufficient(
    req: &RitualReq,
    world: &World,
    players: &PlayerSet,
    participants: &[u32],
    x: u32,
    y: u32,
) -> bool {
    for (i, &need) in req.stones.iter().enumerate() {
        if need == 0 {
            continue;
        }
        let kind = StoneKind::from_index(i).unwrap();
        if stone_count_in_pool(kind, world, players, participants, x, y) < u32::from(need) {
            return false;
        }
    }
    true
}

fn consume_stones(
    req: &RitualReq,
    world: &mut World,
    players: &mut PlayerSet,
    participants: &[u32],
    x: u32,
    y: u32,
) {
    for (i, &need) in req.stones.iter().enumerate() {
        if need == 0 {
            continue;
        }
        let kind = StoneKind::from_index(i).unwrap();
        let mut left = u32::from(need);
        if left > 0 && world.tile(x, y).has_stone(kind) {
            let _ = world.tile_mut(x, y).take_stone(kind);
            left -= 1;
        }
        for &id in participants {
            while left > 0 {
                let Some(p) = players.get_mut(id) else {
                    break;
                };
                if !p.inventory.take_stone(kind) {
                    break;
                }
                left -= 1;
            }
            if left == 0 {
                break;
            }
        }
    }
}

/// Format the subject success line.
pub fn current_level_line(level: u8) -> String {
    format!("current level : {level}\n")
}

pub const EVOLUTION_IN_PROGRESS: &[u8] = b"evolution in progress\n";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player::PlayerSet;
    use crate::world::{SeededRng, StoneKind, World};

    fn place(players: &mut PlayerSet, world: &World, level: u8, x: u32, y: u32) -> u32 {
        let mut rng = SeededRng::new(1);
        let id = players.spawn("t", world, &mut rng);
        let p = players.get_mut(id).unwrap();
        p.x = x;
        p.y = y;
        p.level = level;
        id
    }

    #[test]
    fn table_matches_subject() {
        assert_eq!(RITUAL_TABLE.len(), 7);
        assert_eq!(requirements_for(1).unwrap().players, 1);
        assert_eq!(requirements_for(1).unwrap().stones, [1, 0, 0, 0, 0, 0]);
        assert_eq!(requirements_for(7).unwrap().players, 6);
        assert_eq!(requirements_for(7).unwrap().stones, [2, 2, 2, 2, 2, 1]);
        assert!(requirements_for(8).is_none());
    }

    #[test]
    fn level1_solo_with_jade_on_tile() {
        let mut world = World::empty(5, 5);
        world.tile_mut(1, 1).try_add_stone(StoneKind::Jade);
        let mut players = PlayerSet::default();
        let a = place(&mut players, &world, 1, 1, 1);
        let mut rituals = RitualSet::new();
        let parts = rituals.try_begin(a, &mut players, &world).unwrap();
        assert_eq!(parts, vec![a]);
        assert!(players.get(a).unwrap().in_ritual);

        let (lvl, still) = rituals.try_complete(a, &mut players, &mut world).unwrap();
        assert_eq!(lvl, 2);
        assert_eq!(still, vec![a]);
        assert_eq!(players.get(a).unwrap().level, 2);
        assert!(!players.get(a).unwrap().in_ritual);
        assert!(!world.tile(1, 1).has_stone(StoneKind::Jade));
    }

    #[test]
    fn level1_fails_without_jade() {
        let world = World::empty(5, 5);
        let mut players = PlayerSet::default();
        let a = place(&mut players, &world, 1, 0, 0);
        let mut rituals = RitualSet::new();
        assert!(rituals.try_begin(a, &mut players, &world).is_err());
    }

    #[test]
    fn level2_needs_two_players_and_three_stones() {
        let mut world = World::empty(5, 5);
        world.tile_mut(2, 2).try_add_stone(StoneKind::Jade);
        world.tile_mut(2, 2).try_add_stone(StoneKind::Peridot);
        world.tile_mut(2, 2).try_add_stone(StoneKind::Amber);
        let mut players = PlayerSet::default();
        let a = place(&mut players, &world, 2, 2, 2);
        let b = place(&mut players, &world, 2, 2, 2);
        let mut rituals = RitualSet::new();
        assert!(rituals.try_begin(a, &mut players, &world).is_ok());
        let (lvl, still) = rituals.try_complete(a, &mut players, &mut world).unwrap();
        assert_eq!(lvl, 3);
        assert_eq!(still.len(), 2);
        assert_eq!(players.get(a).unwrap().level, 3);
        assert_eq!(players.get(b).unwrap().level, 3);
    }

    #[test]
    fn inventory_stones_count_toward_pool() {
        let mut world = World::empty(4, 4);
        let mut players = PlayerSet::default();
        let a = place(&mut players, &world, 1, 0, 0);
        players
            .get_mut(a)
            .unwrap()
            .inventory
            .add_stone(StoneKind::Jade);
        let mut rituals = RitualSet::new();
        assert!(rituals.try_begin(a, &mut players, &world).is_ok());
        let _ = rituals.try_complete(a, &mut players, &mut world).unwrap();
        assert_eq!(players.get(a).unwrap().inventory.stone(StoneKind::Jade), 0);
    }

    #[test]
    fn mid_ritual_alone_cancels() {
        let mut world = World::empty(5, 5);
        world.tile_mut(1, 1).try_add_stone(StoneKind::Jade);
        world.tile_mut(1, 1).try_add_stone(StoneKind::Peridot);
        world.tile_mut(1, 1).try_add_stone(StoneKind::Amber);
        let mut players = PlayerSet::default();
        let a = place(&mut players, &world, 2, 1, 1);
        let b = place(&mut players, &world, 2, 1, 1);
        let mut rituals = RitualSet::new();
        rituals.try_begin(a, &mut players, &world).unwrap();
        players.remove(b);
        rituals.on_player_gone(b, &mut players);
        assert!(rituals.get_by_starter(a).is_none());
        assert!(!players.get(a).unwrap().in_ritual);
        assert!(rituals.try_complete(a, &mut players, &mut world).is_err());
        assert_eq!(players.get(a).unwrap().level, 2);
    }

    #[test]
    fn current_level_line_format() {
        assert_eq!(current_level_line(3), "current level : 3\n");
    }
}
