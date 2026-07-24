//! Player vision / `see` formatting (S08 / RQ08 / AQ09).
//!
//! Level `L` sees a forward triangle matching the subject diagrams: tile `0` is
//! the player's own square, then for each depth `d = 1..=L` a row of `2*d+1`
//! tiles listed left-to-right. Total tiles = `(L+1)^2`.
//!
//! The looking player is omitted from their own square. Other objects on a tile
//! are space-separated; empty tiles appear as an empty field between commas.

use crate::player::{Orientation, Player, PlayerSet};
use crate::world::{StoneKind, World};

/// Number of tiles in a `see` response at `level` (subject: level 1→4, 2→9, 3→16).
pub fn vision_tile_count(level: u8) -> usize {
    let l = usize::from(level);
    (l + 1) * (l + 1)
}

/// Absolute map coordinates for each vision slot, in `see` order.
pub fn vision_coords(
    level: u8,
    x: u32,
    y: u32,
    orient: Orientation,
    world: &World,
) -> Vec<(u32, u32)> {
    let mut out = Vec::with_capacity(vision_tile_count(level));
    // Slot 0: current tile.
    out.push((x, y));

    let (fx, fy) = orient.step_delta();
    let (rx, ry) = orient.turn_right().step_delta();

    for depth in 1..=level {
        let d = i64::from(depth);
        // Left → right: lateral offset from -depth to +depth.
        for lateral in -d..=d {
            let wx = i64::from(x) + fx * d + rx * lateral;
            let wy = i64::from(y) + fy * d + ry * lateral;
            out.push((world.wrap_x(wx), world.wrap_y(wy)));
        }
    }
    out
}

/// Format one tile's visible contents (excluding `viewer_id` if on that tile).
pub fn format_tile_contents(
    tx: u32,
    ty: u32,
    viewer_id: u32,
    world: &World,
    players: &PlayerSet,
) -> String {
    let mut parts: Vec<&str> = Vec::new();
    // Players first (subject example: "player amber"), then food, then stones.
    let other_players = players
        .iter()
        .filter(|p| p.id != viewer_id && p.x == tx && p.y == ty)
        .count();
    parts.extend(std::iter::repeat_n("player", other_players));

    let tile = world.tile(tx, ty);
    if tile.food {
        parts.push("food");
    }
    for kind in StoneKind::ALL {
        if tile.has_stone(kind) {
            parts.push(kind.as_str());
        }
    }

    parts.join(" ")
}

/// Full `see` response line including braces and trailing newline.
pub fn see_reply(viewer: &Player, world: &World, players: &PlayerSet) -> String {
    let level = viewer.level.max(1);
    let coords = vision_coords(level, viewer.x, viewer.y, viewer.orient, world);
    let mut body = String::from("{");
    for (i, &(tx, ty)) in coords.iter().enumerate() {
        if i > 0 {
            body.push_str(", ");
        }
        body.push_str(&format_tile_contents(tx, ty, viewer.id, world, players));
    }
    body.push_str("}\n");
    body
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::SeededRng;

    #[test]
    fn tile_counts_match_subject_levels() {
        assert_eq!(vision_tile_count(1), 4);
        assert_eq!(vision_tile_count(2), 9);
        assert_eq!(vision_tile_count(3), 16);
        assert_eq!(vision_tile_count(8), 81);
    }

    #[test]
    fn level1_north_offsets() {
        let world = World::empty(20, 20);
        let coords = vision_coords(1, 10, 10, Orientation::North, &world);
        assert_eq!(coords.len(), 4);
        assert_eq!(coords[0], (10, 10));
        assert_eq!(coords[1], (9, 9));
        assert_eq!(coords[2], (10, 9));
        assert_eq!(coords[3], (11, 9));
    }

    #[test]
    fn vision_grows_with_level() {
        let world = World::empty(30, 30);
        let c1 = vision_coords(1, 15, 15, Orientation::East, &world);
        let c2 = vision_coords(2, 15, 15, Orientation::East, &world);
        assert_eq!(c1.len(), 4);
        assert_eq!(c2.len(), 9);
        assert_eq!(&c2[..4], &c1[..]);
    }

    #[test]
    fn vision_wraps_on_torus() {
        let world = World::empty(5, 5);
        let coords = vision_coords(1, 0, 0, Orientation::West, &world);
        assert_eq!(coords[0], (0, 0));
        assert_eq!(coords[2], (4, 0));
    }

    #[test]
    fn see_omits_self_and_formats_resources() {
        let mut world = World::empty(8, 8);
        world.tile_mut(3, 3).try_add_food();
        world.tile_mut(2, 2).try_add_stone(StoneKind::Amber);
        world.tile_mut(4, 2).try_add_stone(StoneKind::Garnet);
        world.tile_mut(4, 2).try_add_stone(StoneKind::Jade);

        let mut rng = SeededRng::new(1);
        let mut players = PlayerSet::new();
        let vid = players.spawn("a", &world, &mut rng);
        let oid = players.spawn("b", &world, &mut rng);
        {
            let v = players.get_mut(vid).unwrap();
            v.x = 3;
            v.y = 3;
            v.orient = Orientation::North;
            v.level = 1;
        }
        {
            let o = players.get_mut(oid).unwrap();
            o.x = 2;
            o.y = 2;
        }

        let reply = see_reply(players.get(vid).unwrap(), &world, &players);
        let inner = reply.trim_start_matches('{').trim_end_matches("}\n");
        let tiles: Vec<&str> = inner.split(", ").collect();
        assert_eq!(tiles.len(), 4);
        assert_eq!(tiles[0], "food");
        assert_eq!(tiles[1], "player amber");
        assert_eq!(tiles[2], ""); // forward
        assert_eq!(tiles[3], "jade garnet"); // forward-right
    }

    #[test]
    fn empty_level1_see_has_four_empty_slots() {
        let world = World::empty(10, 10);
        let mut rng = SeededRng::new(0);
        let mut players = PlayerSet::new();
        let id = players.spawn("t", &world, &mut rng);
        players.get_mut(id).unwrap().level = 1;
        assert_eq!(
            see_reply(players.get(id).unwrap(), &world, &players),
            "{, , , }\n"
        );
    }
}
