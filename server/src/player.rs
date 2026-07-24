//! Player spawn state (S05) and command queue hook (S06).
//!
//! On a successful team handshake the server creates a [`Player`] with the
//! subject starting loadout (RQ06 / AQ21 / AQ22):
//!
//! - level [`STARTING_LEVEL`] (1)
//! - [`STARTING_FOOD`] (10) food -> [`STARTING_LIFE_TU`] (1260) time units of life
//! - zero of every stone type
//! - membership on the joined team
//! - a random map position and facing (N/E/S/W)
//!
//! Life is stored as remaining time units (`life_tu`). One food unit equals
//! [`FOOD_LIFE_TU`] (126) time units (RQ07 / AQ30). The server drains one TU of
//! life per game time unit (S10); reaching 0 sends `death` and disconnects.

use crate::commands::CmdQueue;
use crate::world::{SeededRng, StoneKind, World};

/// One food unit grants this many time units of life.
pub const FOOD_LIFE_TU: u32 = 126;

/// Food units granted at spawn (converted immediately into life TU).
pub const STARTING_FOOD: u32 = 10;

/// Starting life: 10 food x 126 TU = 1260 (AQ21).
pub const STARTING_LIFE_TU: u32 = STARTING_FOOD * FOOD_LIFE_TU;

/// Every new player begins at this level (AQ22).
pub const STARTING_LEVEL: u8 = 1;

/// Cardinal facing used by movement / kick / vision later.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Orientation {
    North,
    East,
    South,
    West,
}

impl Orientation {
    pub const ALL: [Orientation; 4] = [
        Orientation::North,
        Orientation::East,
        Orientation::South,
        Orientation::West,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Orientation::North => "N",
            Orientation::East => "E",
            Orientation::South => "S",
            Orientation::West => "W",
        }
    }

    /// Turn 90 degrees clockwise (`right` command).
    pub fn turn_right(self) -> Self {
        match self {
            Orientation::North => Orientation::East,
            Orientation::East => Orientation::South,
            Orientation::South => Orientation::West,
            Orientation::West => Orientation::North,
        }
    }

    /// Turn 90 degrees counter-clockwise (`left` command).
    pub fn turn_left(self) -> Self {
        match self {
            Orientation::North => Orientation::West,
            Orientation::West => Orientation::South,
            Orientation::South => Orientation::East,
            Orientation::East => Orientation::North,
        }
    }

    /// One-step delta `(dx, dy)` when advancing while facing this way.
    ///
    /// Grid convention: `+x` east, `+y` south (row-major tiles).
    pub fn step_delta(self) -> (i64, i64) {
        match self {
            Orientation::North => (0, -1),
            Orientation::East => (1, 0),
            Orientation::South => (0, 1),
            Orientation::West => (-1, 0),
        }
    }
}

/// Carried resources. `life_tu` is survival time left (from food).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Inventory {
    /// Remaining life in time units (starts at [`STARTING_LIFE_TU`]).
    pub life_tu: u32,
    pub jade: u32,
    pub peridot: u32,
    pub amber: u32,
    pub amethyst: u32,
    pub garnet: u32,
    pub ammolite: u32,
}

impl Inventory {
    /// Subject starting bag: 10 food -> 1260 TU life, no stones.
    pub fn starting() -> Self {
        Self {
            life_tu: STARTING_LIFE_TU,
            jade: 0,
            peridot: 0,
            amber: 0,
            amethyst: 0,
            garnet: 0,
            ammolite: 0,
        }
    }

    pub fn stone(&self, kind: StoneKind) -> u32 {
        match kind {
            StoneKind::Jade => self.jade,
            StoneKind::Peridot => self.peridot,
            StoneKind::Amber => self.amber,
            StoneKind::Amethyst => self.amethyst,
            StoneKind::Garnet => self.garnet,
            StoneKind::Ammolite => self.ammolite,
        }
    }

    pub fn total_stones(&self) -> u32 {
        self.jade + self.peridot + self.amber + self.amethyst + self.garnet + self.ammolite
    }

    pub fn add_stone(&mut self, kind: StoneKind) {
        match kind {
            StoneKind::Jade => self.jade += 1,
            StoneKind::Peridot => self.peridot += 1,
            StoneKind::Amber => self.amber += 1,
            StoneKind::Amethyst => self.amethyst += 1,
            StoneKind::Garnet => self.garnet += 1,
            StoneKind::Ammolite => self.ammolite += 1,
        }
    }

    /// Remove one stone of `kind`. Returns `false` if none left.
    pub fn take_stone(&mut self, kind: StoneKind) -> bool {
        let slot = match kind {
            StoneKind::Jade => &mut self.jade,
            StoneKind::Peridot => &mut self.peridot,
            StoneKind::Amber => &mut self.amber,
            StoneKind::Amethyst => &mut self.amethyst,
            StoneKind::Garnet => &mut self.garnet,
            StoneKind::Ammolite => &mut self.ammolite,
        };
        if *slot == 0 {
            return false;
        }
        *slot -= 1;
        true
    }

    /// Gain one food unit of life (+[`FOOD_LIFE_TU`]).
    pub fn add_food_life(&mut self) {
        self.life_tu = self.life_tu.saturating_add(FOOD_LIFE_TU);
    }

    /// Spend one food unit of life (-[`FOOD_LIFE_TU`]). Returns `false` if not enough.
    pub fn take_food_life(&mut self) -> bool {
        if self.life_tu < FOOD_LIFE_TU {
            return false;
        }
        self.life_tu -= FOOD_LIFE_TU;
        true
    }
}

/// In-world player created when a client finishes the team handshake.
#[derive(Debug)]
pub struct Player {
    pub id: u32,
    pub team: String,
    pub x: u32,
    pub y: u32,
    pub orient: Orientation,
    pub level: u8,
    pub inventory: Inventory,
    /// True while this player participates in an evolution ritual (S14).
    /// Kick is forbidden on a tile while any occupant has this set (S11 / RQ14).
    pub in_ritual: bool,
    /// Successful requests awaiting a response (S06 / RQ12).
    pub queue: CmdQueue,
}

impl Player {
    /// Spawn on a random tile of `world`, facing a random direction.
    pub fn spawn(id: u32, team: impl Into<String>, world: &World, rng: &mut SeededRng) -> Self {
        let x = rng.gen_index(world.width() as usize) as u32;
        let y = rng.gen_index(world.height() as usize) as u32;
        let orient = Orientation::ALL[rng.gen_index(Orientation::ALL.len())];
        Self {
            id,
            team: team.into(),
            x,
            y,
            orient,
            level: STARTING_LEVEL,
            inventory: Inventory::starting(),
            in_ritual: false,
            queue: CmdQueue::new(),
        }
    }

    /// Drain `tu` life units. Returns `true` when life reaches 0 (starvation).
    pub fn tick_life(&mut self, tu: u32) -> bool {
        if tu == 0 {
            return self.inventory.life_tu == 0;
        }
        if tu >= self.inventory.life_tu {
            self.inventory.life_tu = 0;
            true
        } else {
            self.inventory.life_tu -= tu;
            false
        }
    }

    /// Format the subject `inventory` response line (food = remaining life TU).
    pub fn inventory_reply(&self) -> String {
        let i = &self.inventory;
        format!(
            "{{food {}, jade {}, peridot {}, amber {}, amethyst {}, garnet {}, ammolite {}}}\n",
            i.life_tu, i.jade, i.peridot, i.amber, i.amethyst, i.garnet, i.ammolite
        )
    }

    /// Move one tile forward; coordinates wrap on the torus (RQ03 / AQ10).
    pub fn advance(&mut self, world: &World) {
        let (dx, dy) = self.orient.step_delta();
        self.x = world.wrap_x(i64::from(self.x) + dx);
        self.y = world.wrap_y(i64::from(self.y) + dy);
    }

    /// Face 90 degrees to the right.
    pub fn turn_right(&mut self) {
        self.orient = self.orient.turn_right();
    }

    /// Face 90 degrees to the left.
    pub fn turn_left(&mut self) {
        self.orient = self.orient.turn_left();
    }

    /// Pick `object` from the current tile into inventory (`ok` / `ko`).
    ///
    /// `food` adds [`FOOD_LIFE_TU`] life; stones increment the matching count.
    pub fn pick_object(&mut self, object: &str, world: &mut World) -> bool {
        let tile = world.tile_mut(self.x, self.y);
        if object == "food" {
            if !tile.take_food() {
                return false;
            }
            self.inventory.add_food_life();
            return true;
        }
        let Some(kind) = StoneKind::parse(object) else {
            return false;
        };
        if !tile.take_stone(kind) {
            return false;
        }
        self.inventory.add_stone(kind);
        true
    }

    /// Drop `object` from inventory onto the current tile (`ok` / `ko`).
    ///
    /// Honors tile rules: at most one food; at most one of each stone type;
    /// at most three stone kinds per tile.
    pub fn drop_object(&mut self, object: &str, world: &mut World) -> bool {
        if object == "food" {
            if !self.inventory.take_food_life() {
                return false;
            }
            if !world.tile_mut(self.x, self.y).try_add_food() {
                // Restore life if the tile already has food.
                self.inventory.add_food_life();
                return false;
            }
            return true;
        }
        let Some(kind) = StoneKind::parse(object) else {
            return false;
        };
        if !self.inventory.take_stone(kind) {
            return false;
        }
        if !world.tile_mut(self.x, self.y).try_add_stone(kind) {
            self.inventory.add_stone(kind);
            return false;
        }
        true
    }

    /// True when this player still matches the subject start loadout checks.
    pub fn has_starting_loadout(&self) -> bool {
        self.level == STARTING_LEVEL
            && self.inventory.life_tu == STARTING_LIFE_TU
            && self.inventory.total_stones() == 0
    }
}

/// Registry of live players keyed by id.
#[derive(Debug, Default)]
pub struct PlayerSet {
    next_id: u32,
    players: std::collections::HashMap<u32, Player>,
}

impl PlayerSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.players.len()
    }

    pub fn is_empty(&self) -> bool {
        self.players.is_empty()
    }

    pub fn get(&self, id: u32) -> Option<&Player> {
        self.players.get(&id)
    }

    pub fn get_mut(&mut self, id: u32) -> Option<&mut Player> {
        self.players.get_mut(&id)
    }

    /// Allocate an id, spawn, and insert. Returns the new player id.
    pub fn spawn(&mut self, team: impl Into<String>, world: &World, rng: &mut SeededRng) -> u32 {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        let player = Player::spawn(id, team, world, rng);
        self.players.insert(id, player);
        id
    }

    pub fn remove(&mut self, id: u32) -> Option<Player> {
        self.players.remove(&id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Player> {
        self.players.values()
    }

    pub fn count_on_team(&self, team: &str) -> usize {
        self.players.values().filter(|p| p.team == team).count()
    }

    /// Earliest command completion deadline among busy players (for poll timeout).
    pub fn next_busy_deadline(&self) -> Option<std::time::Instant> {
        self.players
            .values()
            .filter_map(|p| p.queue.busy_until())
            .min()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starting_constants_match_subject() {
        assert_eq!(FOOD_LIFE_TU, 126);
        assert_eq!(STARTING_FOOD, 10);
        assert_eq!(STARTING_LIFE_TU, 1260);
        assert_eq!(STARTING_LEVEL, 1);
    }

    #[test]
    fn starting_inventory_is_ten_food_zero_stones() {
        let inv = Inventory::starting();
        assert_eq!(inv.life_tu, 1260);
        assert_eq!(inv.total_stones(), 0);
        for kind in StoneKind::ALL {
            assert_eq!(inv.stone(kind), 0);
        }
    }

    #[test]
    fn inventory_reply_lists_life_and_stones() {
        let world = World::empty(2, 2);
        let mut rng = SeededRng::new(1);
        let p = Player::spawn(0, "t", &world, &mut rng);
        let reply = p.inventory_reply();
        assert!(reply.starts_with("{food 1260,"));
        assert!(reply.contains("jade 0"));
        assert!(reply.ends_with("}\n"));
    }

    #[test]
    fn spawn_sets_level_team_and_loadout() {
        let world = World::empty(8, 6);
        let mut rng = SeededRng::new(1);
        let p = Player::spawn(0, "alpha", &world, &mut rng);
        assert_eq!(p.id, 0);
        assert_eq!(p.team, "alpha");
        assert_eq!(p.level, 1);
        assert!(p.has_starting_loadout());
        assert!(p.x < 8 && p.y < 6);
        assert!(Orientation::ALL.contains(&p.orient));
        assert_eq!(p.queue.pending_count(), 0);
    }

    #[test]
    fn player_set_spawn_and_remove_tracks_team() {
        let world = World::empty(4, 4);
        let mut rng = SeededRng::new(9);
        let mut set = PlayerSet::new();
        let a = set.spawn("red", &world, &mut rng);
        let b = set.spawn("red", &world, &mut rng);
        let c = set.spawn("blue", &world, &mut rng);
        assert_eq!(set.len(), 3);
        assert_eq!(set.count_on_team("red"), 2);
        assert_eq!(set.count_on_team("blue"), 1);
        assert!(set.get(a).unwrap().has_starting_loadout());
        assert!(set.get(b).unwrap().has_starting_loadout());
        assert_eq!(set.get(c).unwrap().team, "blue");
        set.remove(a);
        assert_eq!(set.count_on_team("red"), 1);
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn turns_cycle_cardinals() {
        assert_eq!(Orientation::North.turn_right(), Orientation::East);
        assert_eq!(Orientation::East.turn_right(), Orientation::South);
        assert_eq!(Orientation::South.turn_right(), Orientation::West);
        assert_eq!(Orientation::West.turn_right(), Orientation::North);

        assert_eq!(Orientation::North.turn_left(), Orientation::West);
        assert_eq!(Orientation::West.turn_left(), Orientation::South);
        assert_eq!(Orientation::South.turn_left(), Orientation::East);
        assert_eq!(Orientation::East.turn_left(), Orientation::North);
    }

    #[test]
    fn advance_moves_forward_and_wraps_toroidally() {
        let world = World::empty(5, 4);
        let mut rng = SeededRng::new(1);
        let mut p = Player::spawn(0, "t", &world, &mut rng);
        p.x = 4;
        p.y = 2;
        p.orient = Orientation::East;
        p.advance(&world);
        assert_eq!((p.x, p.y), (0, 2)); // right edge -> left (AQ10)

        p.orient = Orientation::West;
        p.advance(&world);
        assert_eq!((p.x, p.y), (4, 2));

        p.x = 1;
        p.y = 0;
        p.orient = Orientation::North;
        p.advance(&world);
        assert_eq!((p.x, p.y), (1, 3)); // top -> bottom

        p.orient = Orientation::South;
        p.advance(&world);
        assert_eq!((p.x, p.y), (1, 0));
    }

    #[test]
    fn left_and_right_only_change_facing() {
        let world = World::empty(3, 3);
        let mut rng = SeededRng::new(2);
        let mut p = Player::spawn(0, "t", &world, &mut rng);
        p.x = 1;
        p.y = 1;
        p.orient = Orientation::North;
        p.turn_right();
        assert_eq!(p.orient, Orientation::East);
        assert_eq!((p.x, p.y), (1, 1));
        p.turn_left();
        assert_eq!(p.orient, Orientation::North);
        assert_eq!((p.x, p.y), (1, 1));
    }

    #[test]
    fn spawn_positions_stay_inside_world() {
        let world = World::empty(3, 5);
        let mut rng = SeededRng::new(42);
        let mut set = PlayerSet::new();
        for _ in 0..50 {
            let id = set.spawn("t", &world, &mut rng);
            let p = set.get(id).unwrap();
            assert!(p.x < 3 && p.y < 5);
        }
    }

    #[test]
    fn pick_and_drop_food_and_stones() {
        let mut world = World::empty(4, 4);
        world.tile_mut(1, 1).try_add_food();
        world.tile_mut(1, 1).try_add_stone(StoneKind::Amber);

        let mut rng = SeededRng::new(3);
        let mut p = Player::spawn(0, "t", &world, &mut rng);
        p.x = 1;
        p.y = 1;

        assert!(p.pick_object("food", &mut world));
        assert_eq!(p.inventory.life_tu, STARTING_LIFE_TU + FOOD_LIFE_TU);
        assert!(!world.tile(1, 1).food);
        assert!(!p.pick_object("food", &mut world));

        assert!(p.pick_object("amber", &mut world));
        assert_eq!(p.inventory.amber, 1);
        assert!(!world.tile(1, 1).has_stone(StoneKind::Amber));

        assert!(p.drop_object("amber", &mut world));
        assert_eq!(p.inventory.amber, 0);
        assert!(world.tile(1, 1).has_stone(StoneKind::Amber));

        assert!(p.drop_object("food", &mut world));
        assert_eq!(p.inventory.life_tu, STARTING_LIFE_TU);
        assert!(world.tile(1, 1).food);

        assert!(!p.pick_object("nope", &mut world));
        assert!(!p.drop_object("jade", &mut world));
    }

    #[test]
    fn tick_life_starves_at_zero() {
        let world = World::empty(2, 2);
        let mut rng = SeededRng::new(1);
        let mut p = Player::spawn(0, "t", &world, &mut rng);
        assert!(!p.tick_life(100));
        assert_eq!(p.inventory.life_tu, STARTING_LIFE_TU - 100);
        assert!(p.tick_life(STARTING_LIFE_TU));
        assert_eq!(p.inventory.life_tu, 0);
        assert!(p.tick_life(1));
    }

    #[test]
    fn eating_extends_life_before_starvation() {
        let world = World::empty(2, 2);
        let mut rng = SeededRng::new(1);
        let mut p = Player::spawn(0, "t", &world, &mut rng);
        p.inventory.life_tu = 50;
        p.inventory.add_food_life();
        assert_eq!(p.inventory.life_tu, 50 + FOOD_LIFE_TU);
        assert!(!p.tick_life(100));
        assert_eq!(p.inventory.life_tu, 50 + FOOD_LIFE_TU - 100);
    }
}
