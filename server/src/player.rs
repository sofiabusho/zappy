//! Player spawn state (S05).
//!
//! On a successful team handshake the server creates a [`Player`] with the
//! subject starting loadout (RQ06 / AQ21 / AQ22):
//!
//! - level [`STARTING_LEVEL`] (1)
//! - [`STARTING_FOOD`] (10) food → [`STARTING_LIFE_TU`] (1260) time units of life
//! - zero of every stone type
//! - membership on the joined team
//! - a random map position and facing (N/E/S/W)
//!
//! Life is stored as remaining time units (`life_tu`). One food unit equals
//! [`FOOD_LIFE_TU`] (126) time units (RQ07 / AQ30); S10 will consume this over
//! time. The later `inventory` command reports `food` as this remaining life.

use crate::world::{SeededRng, StoneKind, World};

/// One food unit grants this many time units of life.
pub const FOOD_LIFE_TU: u32 = 126;

/// Food units granted at spawn (converted immediately into life TU).
pub const STARTING_FOOD: u32 = 10;

/// Starting life: 10 food × 126 TU = 1260 (AQ21).
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
    /// Subject starting bag: 10 food → 1260 TU life, no stones.
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
}

/// In-world player created when a client finishes the team handshake.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Player {
    pub id: u32,
    pub team: String,
    pub x: u32,
    pub y: u32,
    pub orient: Orientation,
    pub level: u8,
    pub inventory: Inventory,
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
        }
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
}
