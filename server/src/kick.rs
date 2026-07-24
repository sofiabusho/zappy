//! `kick` command (S11 / RQ14).
//!
//! Pushes every other player on the kicker's tile one step in the kicker's
//! facing direction. Food and stones stay put. Blocked while any player on
//! that tile is in an evolution ritual (`in_ritual`). Victims receive
//! `moving <K>\n` where K is the push direction relative to the victim's
//! facing (same odd sectors as broadcast: 1 front, 3 right, 5 back, 7 left).

use crate::player::{Orientation, PlayerSet};
use crate::world::World;

/// Outcome of completing a `kick` action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KickOutcome {
    /// At least one victim moved; each entry is `(player_id, K)`.
    Ok { victims: Vec<(u32, u8)> },
    /// Nobody to kick, or ritual blocks the action.
    Ko,
}

/// Map absolute push direction to subject sector K relative to `victim_facing`.
pub fn moving_k(victim_facing: Orientation, push: Orientation) -> u8 {
    let v = orient_index(victim_facing);
    let p = orient_index(push);
    match (p + 4 - v) % 4 {
        0 => 1, // forward
        1 => 3, // right
        2 => 5, // behind
        3 => 7, // left
        _ => unreachable!(),
    }
}

fn orient_index(o: Orientation) -> u8 {
    match o {
        Orientation::North => 0,
        Orientation::East => 1,
        Orientation::South => 2,
        Orientation::West => 3,
    }
}

/// Apply `kick` for `kicker_id`. Does not mutate tile resources.
pub fn apply_kick(kicker_id: u32, players: &mut PlayerSet, world: &World) -> KickOutcome {
    let Some(kicker) = players.get(kicker_id) else {
        return KickOutcome::Ko;
    };
    let kx = kicker.x;
    let ky = kicker.y;
    let push = kicker.orient;

    if kicker.in_ritual {
        return KickOutcome::Ko;
    }

    let victim_ids: Vec<u32> = players
        .iter()
        .filter(|p| p.id != kicker_id && p.x == kx && p.y == ky)
        .map(|p| p.id)
        .collect();

    if victim_ids.is_empty() {
        return KickOutcome::Ko;
    }

    if victim_ids
        .iter()
        .any(|&id| players.get(id).is_some_and(|p| p.in_ritual))
    {
        return KickOutcome::Ko;
    }

    let (dx, dy) = push.step_delta();
    let mut victims = Vec::with_capacity(victim_ids.len());
    for id in victim_ids {
        let Some(victim) = players.get_mut(id) else {
            continue;
        };
        let k = moving_k(victim.orient, push);
        victim.x = world.wrap_x(i64::from(victim.x) + dx);
        victim.y = world.wrap_y(i64::from(victim.y) + dy);
        victims.push((id, k));
    }

    if victims.is_empty() {
        KickOutcome::Ko
    } else {
        KickOutcome::Ok { victims }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player::PlayerSet;
    use crate::world::{SeededRng, World};

    fn place(players: &mut PlayerSet, team: &str, x: u32, y: u32, orient: Orientation) -> u32 {
        let world = World::empty(10, 10);
        let mut rng = SeededRng::new(1);
        let id = players.spawn(team, &world, &mut rng);
        let p = players.get_mut(id).unwrap();
        p.x = x;
        p.y = y;
        p.orient = orient;
        id
    }

    #[test]
    fn moving_k_cardinals_relative_to_facing() {
        assert_eq!(moving_k(Orientation::North, Orientation::North), 1);
        assert_eq!(moving_k(Orientation::North, Orientation::East), 3);
        assert_eq!(moving_k(Orientation::North, Orientation::South), 5);
        assert_eq!(moving_k(Orientation::North, Orientation::West), 7);

        assert_eq!(moving_k(Orientation::East, Orientation::East), 1);
        assert_eq!(moving_k(Orientation::East, Orientation::South), 3);
        assert_eq!(moving_k(Orientation::East, Orientation::West), 5);
        assert_eq!(moving_k(Orientation::East, Orientation::North), 7);
    }

    #[test]
    fn kick_alone_is_ko() {
        let world = World::empty(5, 5);
        let mut players = PlayerSet::default();
        let kicker = place(&mut players, "a", 2, 2, Orientation::North);
        assert_eq!(apply_kick(kicker, &mut players, &world), KickOutcome::Ko);
    }

    #[test]
    fn kick_pushes_victims_and_leaves_resources() {
        let mut world = World::empty(5, 5);
        world.tile_mut(2, 2).try_add_food();
        world
            .tile_mut(2, 2)
            .try_add_stone(crate::world::StoneKind::Jade);

        let mut players = PlayerSet::default();
        let kicker = place(&mut players, "a", 2, 2, Orientation::East);
        let victim = place(&mut players, "b", 2, 2, Orientation::North);

        let food_before = world.tile(2, 2).food;
        let jade_before = world.tile(2, 2).has_stone(crate::world::StoneKind::Jade);

        let outcome = apply_kick(kicker, &mut players, &world);
        assert_eq!(
            outcome,
            KickOutcome::Ok {
                victims: vec![(victim, 3)] // victim faces N, push E → right = 3
            }
        );

        let v = players.get(victim).unwrap();
        assert_eq!((v.x, v.y), (3, 2));
        assert_eq!(players.get(kicker).unwrap().x, 2);
        assert_eq!(players.get(kicker).unwrap().y, 2);

        assert_eq!(world.tile(2, 2).food, food_before);
        assert_eq!(
            world.tile(2, 2).has_stone(crate::world::StoneKind::Jade),
            jade_before
        );
        assert!(!world.tile(3, 2).food);
        assert!(!world.tile(3, 2).has_stone(crate::world::StoneKind::Jade));
    }

    #[test]
    fn kick_wraps_on_torus() {
        let world = World::empty(4, 4);
        let mut players = PlayerSet::default();
        let kicker = place(&mut players, "a", 0, 1, Orientation::West);
        let victim = place(&mut players, "b", 0, 1, Orientation::West);
        let outcome = apply_kick(kicker, &mut players, &world);
        assert!(matches!(outcome, KickOutcome::Ok { .. }));
        let v = players.get(victim).unwrap();
        assert_eq!((v.x, v.y), (3, 1));
    }

    #[test]
    fn kick_blocked_when_kicker_in_ritual() {
        let world = World::empty(5, 5);
        let mut players = PlayerSet::default();
        let kicker = place(&mut players, "a", 1, 1, Orientation::South);
        let _victim = place(&mut players, "b", 1, 1, Orientation::North);
        players.get_mut(kicker).unwrap().in_ritual = true;
        assert_eq!(apply_kick(kicker, &mut players, &world), KickOutcome::Ko);
    }

    #[test]
    fn kick_blocked_when_victim_in_ritual() {
        let world = World::empty(5, 5);
        let mut players = PlayerSet::default();
        let kicker = place(&mut players, "a", 1, 1, Orientation::South);
        let victim = place(&mut players, "b", 1, 1, Orientation::North);
        players.get_mut(victim).unwrap().in_ritual = true;
        assert_eq!(apply_kick(kicker, &mut players, &world), KickOutcome::Ko);
        assert_eq!(players.get(victim).unwrap().y, 1);
    }

    #[test]
    fn kick_moves_all_sharing_tile() {
        let world = World::empty(6, 6);
        let mut players = PlayerSet::default();
        let kicker = place(&mut players, "a", 3, 3, Orientation::North);
        let v1 = place(&mut players, "b", 3, 3, Orientation::South);
        let v2 = place(&mut players, "c", 3, 3, Orientation::East);
        let _elsewhere = place(&mut players, "d", 0, 0, Orientation::North);

        match apply_kick(kicker, &mut players, &world) {
            KickOutcome::Ok { victims } => {
                assert_eq!(victims.len(), 2);
                assert!(victims.iter().any(|(id, _)| *id == v1));
                assert!(victims.iter().any(|(id, _)| *id == v2));
            }
            KickOutcome::Ko => panic!("expected ok"),
        }
        assert_eq!(players.get(v1).unwrap().y, 2);
        assert_eq!(players.get(v2).unwrap().y, 2);
        assert_eq!(players.get(kicker).unwrap().y, 3);
    }
}
