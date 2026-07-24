//! `broadcast` sound direction (S12 / RQ15 / AQ32 / AQ33).
//!
//! Sender gets `ok`. Every other live player receives `message <K>,<text>\n`
//! where K is the shortest-path arrival sector relative to the receiver's
//! facing:
//!
//! ```text
//! 8 1 2
//! 7 0 3
//! 6 5 4
//! ```
//!
//! (1 = front, then clockwise on the map with north-up — matches kick
//! cardinals 1/3/5/7). Same tile → `K = 0`.

use crate::player::{Orientation, Player, PlayerSet};
use crate::world::World;

/// Format the protocol line delivered to one listener.
pub fn message_line(k: u8, text: &str) -> String {
    format!("message {k},{text}\n")
}

/// Sound sector K for a broadcast from `(sx,sy)` heard by `receiver`.
pub fn sound_k(receiver: &Player, sx: u32, sy: u32, world: &World) -> u8 {
    let dx = toroidal_delta(receiver.x, sx, world.width());
    let dy = toroidal_delta(receiver.y, sy, world.height());
    if dx == 0 && dy == 0 {
        return 0;
    }
    let (ldx, ldy) = to_local(dx, dy, receiver.orient);
    local_to_sector(ldx, ldy)
}

/// Signed shortest-axis delta from `from` to `to` on a torus of `size`.
///
/// Positive means `to` is greater than `from` along the axis before wrapping.
/// When both directions are equal length (even `size`, half-map apart), the
/// negative direction is preferred for determinism.
pub fn toroidal_delta(from: u32, to: u32, size: u32) -> i32 {
    debug_assert!(size > 0);
    let size = size as i32;
    let mut d = to as i32 - from as i32;
    if d > size / 2 {
        d -= size;
    } else if d < -(size / 2) {
        d += size;
    }
    if size % 2 == 0 && d == size / 2 {
        d -= size;
    }
    d
}

/// Rotate world delta so receiver facing becomes local north (`-y`).
/// Local `+x` is the receiver's right.
fn to_local(dx: i32, dy: i32, orient: Orientation) -> (i32, i32) {
    match orient {
        Orientation::North => (dx, dy),
        Orientation::East => (dy, -dx),
        Orientation::South => (-dx, -dy),
        Orientation::West => (-dy, dx),
    }
}

/// Map local vector (right = +x, down = +y, front = −y) to sector 1..=8.
fn local_to_sector(dx: i32, dy: i32) -> u8 {
    if dx == 0 && dy == 0 {
        return 0;
    }
    let ax = dx.unsigned_abs();
    let ay = dy.unsigned_abs();
    // Half-sector boundary: tan(22.5°) ≈ 0.414 → keep pure axis within ~22.5°.
    let near_vertical = ax.saturating_mul(1000) < ay.saturating_mul(414);
    let near_horizontal = ay.saturating_mul(1000) < ax.saturating_mul(414);

    if near_vertical {
        if dy < 0 {
            1
        } else {
            5
        }
    } else if near_horizontal {
        if dx > 0 {
            3
        } else {
            7
        }
    } else {
        match (dx > 0, dy < 0) {
            (true, true) => 2,   // NE
            (true, false) => 4,  // SE
            (false, false) => 6, // SW
            (false, true) => 8,  // NW
        }
    }
}

/// Collect `(player_id, K)` for every listener of a broadcast from `sender_id`.
pub fn broadcast_targets(sender_id: u32, players: &PlayerSet, world: &World) -> Vec<(u32, u8)> {
    let Some(sender) = players.get(sender_id) else {
        return Vec::new();
    };
    let sx = sender.x;
    let sy = sender.y;
    players
        .iter()
        .filter(|p| p.id != sender_id)
        .map(|p| (p.id, sound_k(p, sx, sy, world)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player::PlayerSet;
    use crate::world::{SeededRng, World};

    fn place(players: &mut PlayerSet, x: u32, y: u32, orient: Orientation) -> u32 {
        let world = World::empty(20, 20);
        let mut rng = SeededRng::new(1);
        let id = players.spawn("t", &world, &mut rng);
        let p = players.get_mut(id).unwrap();
        p.x = x;
        p.y = y;
        p.orient = orient;
        id
    }

    #[test]
    fn same_tile_is_zero() {
        let world = World::empty(10, 10);
        let mut players = PlayerSet::default();
        let a = place(&mut players, 3, 3, Orientation::North);
        let b = place(&mut players, 3, 3, Orientation::East);
        assert_eq!(sound_k(players.get(b).unwrap(), 3, 3, &world), 0);
        let targets = broadcast_targets(a, &players, &world);
        assert_eq!(targets, vec![(b, 0)]);
    }

    #[test]
    fn cardinals_when_facing_north() {
        let world = World::empty(10, 10);
        let mut players = PlayerSet::default();
        let recv = place(&mut players, 5, 5, Orientation::North);
        let r = players.get(recv).unwrap();
        assert_eq!(sound_k(r, 5, 4, &world), 1); // north of recv
        assert_eq!(sound_k(r, 6, 5, &world), 3); // east
        assert_eq!(sound_k(r, 5, 6, &world), 5); // south
        assert_eq!(sound_k(r, 4, 5, &world), 7); // west
        assert_eq!(sound_k(r, 6, 4, &world), 2); // NE
        assert_eq!(sound_k(r, 6, 6, &world), 4); // SE
        assert_eq!(sound_k(r, 4, 6, &world), 6); // SW
        assert_eq!(sound_k(r, 4, 4, &world), 8); // NW
    }

    #[test]
    fn facing_east_rotates_sectors() {
        let world = World::empty(10, 10);
        let mut players = PlayerSet::default();
        // Facing east: front is +x → world east should be K=1
        let recv = place(&mut players, 5, 5, Orientation::East);
        let r = players.get(recv).unwrap();
        assert_eq!(sound_k(r, 6, 5, &world), 1);
        assert_eq!(sound_k(r, 5, 6, &world), 3); // south = right when facing east
        assert_eq!(sound_k(r, 4, 5, &world), 5); // west = behind
        assert_eq!(sound_k(r, 5, 4, &world), 7); // north = left
    }

    #[test]
    fn chooses_shortest_toroidal_path() {
        let world = World::empty(10, 10);
        let mut players = PlayerSet::default();
        // Receiver at (0,0) facing north; sender at (9,0) → wrap west → K=7
        let recv = place(&mut players, 0, 0, Orientation::North);
        let r = players.get(recv).unwrap();
        assert_eq!(sound_k(r, 9, 0, &world), 7);
        // Sender at (0,9) → wrap north → K=1
        assert_eq!(sound_k(r, 0, 9, &world), 1);
    }

    #[test]
    fn toroidal_delta_prefers_short_side() {
        assert_eq!(toroidal_delta(0, 1, 10), 1);
        assert_eq!(toroidal_delta(0, 9, 10), -1);
        assert_eq!(toroidal_delta(5, 5, 10), 0);
        // Exactly half on even size → prefer negative
        assert_eq!(toroidal_delta(0, 5, 10), -5);
    }

    #[test]
    fn message_line_format() {
        assert_eq!(message_line(2, "hello"), "message 2,hello\n");
        assert_eq!(message_line(0, ""), "message 0,\n");
        assert_eq!(message_line(4, "a b c"), "message 4,a b c\n");
    }

    #[test]
    fn broadcast_skips_sender() {
        let world = World::empty(8, 8);
        let mut players = PlayerSet::default();
        let sender = place(&mut players, 1, 1, Orientation::North);
        let other = place(&mut players, 2, 1, Orientation::North);
        let targets = broadcast_targets(sender, &players, &world);
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].0, other);
        assert_eq!(targets[0].1, 7); // sender is west of other? other at (2,1), sender (1,1)
                                     // sound from sender to other: from other's view sender is at west → 7
    }
}
