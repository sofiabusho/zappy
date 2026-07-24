//! `broadcast` command + directional sound (S12 / RQ15 / AQ32 / AQ33).
//!
//! A player sends `broadcast <text>` and receives `ok`. Every *other* living
//! player receives `message <K>,<text>\n`, where `K` encodes the direction the
//! sound arrives from, relative to that receiver's own facing.
//!
//! ## Direction K
//!
//! Per the subject ("Sound Transmission"): `K = 0` when the sound originates on
//! the receiver's own tile. Otherwise `1` is the square directly in front of the
//! receiver and the remaining squares are numbered **counter-clockwise**
//! (trigonometric direction):
//!
//! ```text
//!      2  1  8
//!      3  P  7      (P faces up = square 1 is "front")
//!      4  5  6
//! ```
//!
//! so the cardinals relative to facing are: front `1`, left `3`, back `5`,
//! right `7`, and the diagonals fill `2/4/6/8`.
//!
//! Because the world is a torus, the sound takes the **shortest** path: on each
//! axis we reduce the offset to its representative closest to zero (the halfway
//! tie on an even dimension resolves to the positive side, deterministically).
//! The reduced `(dx, dy)` is then rotated into the receiver's local frame and
//! bucketed into one of the eight sectors by the signs of its forward / right
//! components.

use crate::player::{Orientation, PlayerSet};
use crate::world::World;

/// One `message <K>,<text>` to deliver to a receiving player.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SoundMessage {
    pub player_id: u32,
    pub k: u8,
}

/// Shortest signed offset of `to - from` on a wrapped axis of length `n`.
///
/// Result lies in `-(n/2) ..= n/2`; an exact halfway split resolves positive.
fn shortest_delta(from: u32, to: u32, n: u32) -> i64 {
    if n == 0 {
        return 0;
    }
    let n = i64::from(n);
    let mut d = (i64::from(to) - i64::from(from)).rem_euclid(n);
    if d > n / 2 {
        d -= n;
    }
    d
}

/// Sector `K` (0..=8) for a sound arriving at a receiver facing `facing` from a
/// source whose shortest toroidal offset is `(dx, dy)` in map coordinates
/// (`+x` east, `+y` south). `(0, 0)` → same tile → `0`.
pub fn direction_k(facing: Orientation, dx: i64, dy: i64) -> u8 {
    if dx == 0 && dy == 0 {
        return 0;
    }

    // Rotate the map-space offset into the receiver's local frame:
    // `forward` > 0 means the source is ahead, `right` > 0 means to the right.
    let (forward, right) = match facing {
        Orientation::North => (-dy, dx),
        Orientation::East => (dx, dy),
        Orientation::South => (dy, -dx),
        Orientation::West => (-dx, -dy),
    };

    // Numbering is counter-clockwise from the front, so "left" (right < 0)
    // takes the low sectors (2, 3, 4) and "right" (right > 0) the high (6, 7, 8).
    match (forward.signum(), right.signum()) {
        (1, 0) => 1,   // front
        (1, -1) => 2,  // front-left
        (0, -1) => 3,  // left
        (-1, -1) => 4, // back-left
        (-1, 0) => 5,  // back
        (-1, 1) => 6,  // back-right
        (0, 1) => 7,   // right
        (1, 1) => 8,   // front-right
        _ => unreachable!("dx/dy both zero handled above"),
    }
}

/// Build the `message <K>,...` deliveries for `broadcast` sent by `source_id`.
///
/// The broadcaster is excluded (players never hear their own broadcast). Order
/// is unspecified; callers deliver each to the matching connection.
pub fn apply_broadcast(source_id: u32, players: &PlayerSet, world: &World) -> Vec<SoundMessage> {
    let Some(source) = players.get(source_id) else {
        return Vec::new();
    };
    let (sx, sy) = (source.x, source.y);
    let width = world.width();
    let height = world.height();

    players
        .iter()
        .filter(|p| p.id != source_id)
        .map(|p| {
            let dx = shortest_delta(p.x, sx, width);
            let dy = shortest_delta(p.y, sy, height);
            SoundMessage {
                player_id: p.id,
                k: direction_k(p.orient, dx, dy),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::{SeededRng, World};

    fn place(players: &mut PlayerSet, world: &World, x: u32, y: u32, orient: Orientation) -> u32 {
        let mut rng = SeededRng::new(7);
        let id = players.spawn("t", world, &mut rng);
        let p = players.get_mut(id).unwrap();
        p.x = x;
        p.y = y;
        p.orient = orient;
        id
    }

    #[test]
    fn same_tile_is_zero() {
        assert_eq!(direction_k(Orientation::North, 0, 0), 0);
        assert_eq!(direction_k(Orientation::East, 0, 0), 0);
    }

    #[test]
    fn cardinals_relative_to_north_facing() {
        // Facing North (up = -y). Source directly ahead is north (dy < 0).
        assert_eq!(direction_k(Orientation::North, 0, -1), 1); // front
        assert_eq!(direction_k(Orientation::North, -1, 0), 3); // left (west)
        assert_eq!(direction_k(Orientation::North, 0, 1), 5); // back (south)
        assert_eq!(direction_k(Orientation::North, 1, 0), 7); // right (east)
    }

    #[test]
    fn diagonals_are_counter_clockwise() {
        // North-facing: front-left, back-left, back-right, front-right.
        assert_eq!(direction_k(Orientation::North, -1, -1), 2);
        assert_eq!(direction_k(Orientation::North, -1, 1), 4);
        assert_eq!(direction_k(Orientation::North, 1, 1), 6);
        assert_eq!(direction_k(Orientation::North, 1, -1), 8);
    }

    #[test]
    fn front_is_one_for_every_facing() {
        assert_eq!(direction_k(Orientation::North, 0, -1), 1);
        assert_eq!(direction_k(Orientation::East, 1, 0), 1);
        assert_eq!(direction_k(Orientation::South, 0, 1), 1);
        assert_eq!(direction_k(Orientation::West, -1, 0), 1);
    }

    #[test]
    fn right_is_seven_for_every_facing() {
        // Source to the receiver's right hand relative to facing.
        assert_eq!(direction_k(Orientation::North, 1, 0), 7); // east
        assert_eq!(direction_k(Orientation::East, 0, 1), 7); // south
        assert_eq!(direction_k(Orientation::South, -1, 0), 7); // west
        assert_eq!(direction_k(Orientation::West, 0, -1), 7); // north
    }

    #[test]
    fn shortest_delta_wraps_to_nearest() {
        // Width 10: from 9 to 0 is +1 (wrap), not -9.
        assert_eq!(shortest_delta(9, 0, 10), 1);
        assert_eq!(shortest_delta(0, 9, 10), -1);
        assert_eq!(shortest_delta(2, 5, 10), 3);
        // Exact halfway resolves positive.
        assert_eq!(shortest_delta(0, 5, 10), 5);
    }

    #[test]
    fn broadcast_uses_shortest_toroidal_path() {
        // 10x10 torus. Source at x=0; receiver at x=9 facing East.
        // Shortest path from receiver(9) to source(0) is +1 (east) → front → 1.
        let world = World::empty(10, 10);
        let mut players = PlayerSet::default();
        let source = place(&mut players, &world, 0, 0, Orientation::North);
        let recv = place(&mut players, &world, 9, 0, Orientation::East);

        let msgs = apply_broadcast(source, &players, &world);
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].player_id, recv);
        assert_eq!(msgs[0].k, 1);
    }

    #[test]
    fn same_tile_receiver_hears_zero() {
        let world = World::empty(8, 8);
        let mut players = PlayerSet::default();
        let source = place(&mut players, &world, 3, 3, Orientation::North);
        let recv = place(&mut players, &world, 3, 3, Orientation::South);
        let msgs = apply_broadcast(source, &players, &world);
        assert_eq!(
            msgs,
            vec![SoundMessage {
                player_id: recv,
                k: 0
            }]
        );
    }

    #[test]
    fn broadcaster_excluded_and_all_others_reached() {
        let world = World::empty(12, 12);
        let mut players = PlayerSet::default();
        let source = place(&mut players, &world, 6, 6, Orientation::North);
        let a = place(&mut players, &world, 6, 3, Orientation::North); // north of src
        let b = place(&mut players, &world, 9, 6, Orientation::North); // east of src

        let msgs = apply_broadcast(source, &players, &world);
        assert_eq!(msgs.len(), 2);
        assert!(msgs.iter().all(|m| m.player_id != source));

        let ka = msgs.iter().find(|m| m.player_id == a).unwrap().k;
        let kb = msgs.iter().find(|m| m.player_id == b).unwrap().k;
        // `a` sits north of the source, so the sound comes from the south =
        // behind a north-facing listener → 5.
        assert_eq!(ka, 5);
        // `b` sits east of the source, sound comes from the west = left of a
        // north-facing listener → 3.
        assert_eq!(kb, 3);
    }

    #[test]
    fn missing_source_yields_no_messages() {
        let world = World::empty(4, 4);
        let players = PlayerSet::default();
        assert!(apply_broadcast(999, &players, &world).is_empty());
    }
}
