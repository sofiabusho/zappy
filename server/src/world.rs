//! Toroidal world grid and resource generation (S04).
//!
//! The map is obstacle-free plains (RQ03). Coordinates wrap on every edge
//! ([`World::wrap_x`] / [`World::wrap_y`]) so leaving the right side re-enters
//! on the left (AQ10).
//!
//! # Resource generation rules (RQ05 / AQ29)
//!
//! Subject hard constraints (always enforced):
//! 1. At most **one food** per tile.
//! 2. At most **one** stone of each type per tile.
//! 3. At most **three** distinct stone types on one tile.
//! 4. Stones are spread across the map (never dumped onto a single tile when
//!    more than three stones are placed).
//!
//! Project density targets (fraction of tiles, applied independently per
//! resource, then clipped by the hard constraints above):
//!
//! | Resource  | Initial density |
//! |-----------|----------------:|
//! | food      |            0.50 |
//! | jade      |            0.30 |
//! | peridot   |            0.25 |
//! | amber     |            0.20 |
//! | amethyst  |            0.15 |
//! | garnet    |            0.10 |
//! | ammolite  |            0.05 |
//!
//! Placement algorithm: for each resource, compute
//! `target = round(density * width * height)`, shuffle tile indices with the
//! seeded RNG, and place on the first eligible tiles (food empty / stone type
//! absent and stone-type count `< 3`).
//!
//! # Respawn (documented for auditors; driven later by the time loop)
//!
//! Every [`RESPAWN_PERIOD_TU`] time units the server should call
//! [`World::respawn_tick`], which attempts to refill each resource toward the
//! same density targets at [`RESPAWN_RATE`] of the missing count (rounded up,
//! at least one attempt when anything is missing). Respawn obeys the same
//! hard constraints as initial generation.

use std::fmt;

/// Time units between respawn refill passes (wired in by S06+).
pub const RESPAWN_PERIOD_TU: u32 = 20;

/// Fraction of the *missing* count toward density to try placing each respawn.
pub const RESPAWN_RATE: f64 = 0.15;

/// Initial / target densities (see module docs).
pub const FOOD_DENSITY: f64 = 0.50;

/// Density per stone kind, indexed like [`StoneKind`] discriminant order.
pub const STONE_DENSITIES: [f64; 6] = [0.30, 0.25, 0.20, 0.15, 0.10, 0.05];

/// Maximum distinct stone types allowed on one tile (subject rule).
pub const MAX_STONE_TYPES_PER_TILE: usize = 3;

/// The six stone types from the subject (RQ04 / AQ28).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum StoneKind {
    Jade = 0,
    Peridot = 1,
    Amber = 2,
    Amethyst = 3,
    Garnet = 4,
    Ammolite = 5,
}

impl StoneKind {
    pub const ALL: [StoneKind; 6] = [
        StoneKind::Jade,
        StoneKind::Peridot,
        StoneKind::Amber,
        StoneKind::Amethyst,
        StoneKind::Garnet,
        StoneKind::Ammolite,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            StoneKind::Jade => "jade",
            StoneKind::Peridot => "peridot",
            StoneKind::Amber => "amber",
            StoneKind::Amethyst => "amethyst",
            StoneKind::Garnet => "garnet",
            StoneKind::Ammolite => "ammolite",
        }
    }

    pub fn from_index(i: usize) -> Option<StoneKind> {
        Self::ALL.get(i).copied()
    }

    /// Parse a subject stone name (`jade`, …).
    pub fn parse(name: &str) -> Option<StoneKind> {
        Self::ALL.into_iter().find(|k| k.as_str() == name)
    }
}

impl fmt::Display for StoneKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One map square: plains tile with optional food and up to three stone kinds.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Tile {
    /// `true` when this tile holds the single allowed food unit.
    pub food: bool,
    /// Presence of each stone kind (at most one of each; ≤3 kinds total).
    stones: [bool; 6],
}

impl Tile {
    pub fn stone_count(&self) -> usize {
        self.stones.iter().filter(|&&s| s).count()
    }

    pub fn has_stone(&self, kind: StoneKind) -> bool {
        self.stones[kind as usize]
    }

    pub fn stones(&self) -> impl Iterator<Item = StoneKind> + '_ {
        StoneKind::ALL
            .into_iter()
            .filter(|&k| self.stones[k as usize])
    }

    /// Try to place food. Returns `false` if food is already present.
    pub fn try_add_food(&mut self) -> bool {
        if self.food {
            return false;
        }
        self.food = true;
        true
    }

    /// Try to place a stone kind. Honors one-per-type and max-three-kinds.
    pub fn try_add_stone(&mut self, kind: StoneKind) -> bool {
        let i = kind as usize;
        if self.stones[i] {
            return false;
        }
        if self.stone_count() >= MAX_STONE_TYPES_PER_TILE {
            return false;
        }
        self.stones[i] = true;
        true
    }

    pub fn take_food(&mut self) -> bool {
        if self.food {
            self.food = false;
            true
        } else {
            false
        }
    }

    pub fn take_stone(&mut self, kind: StoneKind) -> bool {
        let i = kind as usize;
        if self.stones[i] {
            self.stones[i] = false;
            true
        } else {
            false
        }
    }
}

/// Tiny xorshift64* RNG so generation is seedable without an extra crate (B04).
#[derive(Debug, Clone)]
pub struct SeededRng {
    state: u64,
}

impl SeededRng {
    pub fn new(seed: u64) -> Self {
        // Avoid the all-zero fixed point of xorshift.
        Self {
            state: if seed == 0 {
                0xDEAD_BEEF_CAFE_BABE
            } else {
                seed
            },
        }
    }

    pub fn from_entropy() -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(1);
        Self::new(nanos ^ 0xA5A5_5A5A_C3C3_3C3C)
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    /// Uniform index in `0..len` (`len` must be > 0).
    pub fn gen_index(&mut self, len: usize) -> usize {
        debug_assert!(len > 0);
        (self.next_u64() as usize) % len
    }

    /// Fisher–Yates shuffle.
    pub fn shuffle<T>(&mut self, slice: &mut [T]) {
        for i in (1..slice.len()).rev() {
            let j = self.gen_index(i + 1);
            slice.swap(i, j);
        }
    }
}

/// Toroidal resource map.
#[derive(Debug, Clone)]
pub struct World {
    width: u32,
    height: u32,
    tiles: Vec<Tile>,
    /// Seed used for the last full generation (for logs / reproducibility).
    pub seed: u64,
}

impl World {
    /// Empty plains map (no resources yet).
    pub fn empty(width: u32, height: u32) -> Self {
        assert!(width > 0 && height > 0, "world dimensions must be > 0");
        let n = (width as usize)
            .checked_mul(height as usize)
            .expect("world too large");
        Self {
            width,
            height,
            tiles: vec![Tile::default(); n],
            seed: 0,
        }
    }

    /// Build a world and run initial resource generation with `seed`.
    pub fn generate(width: u32, height: u32, seed: u64) -> Self {
        let mut world = Self::empty(width, height);
        world.seed = seed;
        let mut rng = SeededRng::new(seed);
        world.populate_initial(&mut rng);
        world
    }

    /// Generate with entropy-derived seed.
    pub fn generate_random(width: u32, height: u32) -> Self {
        let rng_seed = SeededRng::from_entropy().state;
        Self::generate(width, height, rng_seed)
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn tile_count(&self) -> usize {
        self.tiles.len()
    }

    /// Wrap a possibly-negative / out-of-range X onto the torus.
    pub fn wrap_x(&self, x: i64) -> u32 {
        wrap_coord(x, self.width as i64) as u32
    }

    /// Wrap a possibly-negative / out-of-range Y onto the torus.
    pub fn wrap_y(&self, y: i64) -> u32 {
        wrap_coord(y, self.height as i64) as u32
    }

    fn index(&self, x: u32, y: u32) -> usize {
        debug_assert!(x < self.width && y < self.height);
        (y as usize) * (self.width as usize) + (x as usize)
    }

    pub fn tile(&self, x: u32, y: u32) -> &Tile {
        &self.tiles[self.index(x, y)]
    }

    pub fn tile_mut(&mut self, x: u32, y: u32) -> &mut Tile {
        let i = self.index(x, y);
        &mut self.tiles[i]
    }

    /// Toroidal neighbour lookup (for movement / kick later).
    pub fn tile_at_wrapped(&self, x: i64, y: i64) -> &Tile {
        self.tile(self.wrap_x(x), self.wrap_y(y))
    }

    pub fn count_food(&self) -> usize {
        self.tiles.iter().filter(|t| t.food).count()
    }

    pub fn count_stone(&self, kind: StoneKind) -> usize {
        self.tiles.iter().filter(|t| t.has_stone(kind)).count()
    }

    pub fn count_all_stones(&self) -> usize {
        StoneKind::ALL.iter().map(|&k| self.count_stone(k)).sum()
    }

    fn target_count(density: f64, tiles: usize) -> usize {
        ((density * tiles as f64).round() as usize).min(tiles)
    }

    fn populate_initial(&mut self, rng: &mut SeededRng) {
        let n = self.tile_count();
        self.place_food(Self::target_count(FOOD_DENSITY, n), rng);
        for kind in StoneKind::ALL {
            let density = STONE_DENSITIES[kind as usize];
            self.place_stone(kind, Self::target_count(density, n), rng);
        }
    }

    fn place_food(&mut self, target: usize, rng: &mut SeededRng) {
        let mut order: Vec<usize> = (0..self.tile_count()).collect();
        rng.shuffle(&mut order);
        let mut placed = 0;
        for i in order {
            if placed >= target {
                break;
            }
            if self.tiles[i].try_add_food() {
                placed += 1;
            }
        }
    }

    fn place_stone(&mut self, kind: StoneKind, target: usize, rng: &mut SeededRng) {
        let mut order: Vec<usize> = (0..self.tile_count()).collect();
        rng.shuffle(&mut order);
        let mut placed = 0;
        for i in order {
            if placed >= target {
                break;
            }
            if self.tiles[i].try_add_stone(kind) {
                placed += 1;
            }
        }
    }

    /// Soft refill toward density targets (respawn policy).
    pub fn respawn_tick(&mut self, rng: &mut SeededRng) {
        let n = self.tile_count();

        let food_target = Self::target_count(FOOD_DENSITY, n);
        let food_have = self.count_food();
        if food_have < food_target {
            let missing = food_target - food_have;
            let attempts = ((missing as f64) * RESPAWN_RATE).ceil() as usize;
            self.place_food(attempts.max(1).min(missing), rng);
        }

        for kind in StoneKind::ALL {
            let target = Self::target_count(STONE_DENSITIES[kind as usize], n);
            let have = self.count_stone(kind);
            if have < target {
                let missing = target - have;
                let attempts = ((missing as f64) * RESPAWN_RATE).ceil() as usize;
                self.place_stone(kind, attempts.max(1).min(missing), rng);
            }
        }
    }

    /// Human-readable summary for server logs / auditors.
    pub fn summary_line(&self) -> String {
        let mut parts = vec![format!("food={}", self.count_food())];
        for kind in StoneKind::ALL {
            parts.push(format!("{}={}", kind.as_str(), self.count_stone(kind)));
        }
        format!(
            "world {}x{} seed={} tiles={} {}",
            self.width,
            self.height,
            self.seed,
            self.tile_count(),
            parts.join(" ")
        )
    }

    /// Validate hard subject constraints across all tiles.
    pub fn assert_invariants(&self) {
        for tile in &self.tiles {
            assert!(tile.stone_count() <= MAX_STONE_TYPES_PER_TILE);
            // food is bool — at most one by construction.
            for kind in StoneKind::ALL {
                let _ = tile.has_stone(kind); // 0 or 1 by construction
            }
        }
    }
}

/// Euclidean-modulo wrap into `0..dim`.
fn wrap_coord(value: i64, dim: i64) -> i64 {
    debug_assert!(dim > 0);
    let m = value % dim;
    if m < 0 {
        m + dim
    } else {
        m
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_six_stone_names_match_subject() {
        assert_eq!(
            StoneKind::ALL.map(|k| k.as_str()),
            ["jade", "peridot", "amber", "amethyst", "garnet", "ammolite"]
        );
    }

    #[test]
    fn wrap_x_leaves_right_enters_left() {
        let w = World::empty(10, 5);
        assert_eq!(w.wrap_x(10), 0); // past right edge
        assert_eq!(w.wrap_x(11), 1);
        assert_eq!(w.wrap_x(-1), 9); // past left edge
        assert_eq!(w.wrap_x(0), 0);
        assert_eq!(w.wrap_x(9), 9);
    }

    #[test]
    fn wrap_y_is_toroidal() {
        let w = World::empty(3, 7);
        assert_eq!(w.wrap_y(7), 0);
        assert_eq!(w.wrap_y(-1), 6);
        assert_eq!(w.wrap_y(100), 2); // 100 % 7 = 2
    }

    #[test]
    fn tile_rejects_second_food_and_fourth_stone_kind() {
        let mut t = Tile::default();
        assert!(t.try_add_food());
        assert!(!t.try_add_food());

        assert!(t.try_add_stone(StoneKind::Jade));
        assert!(t.try_add_stone(StoneKind::Peridot));
        assert!(t.try_add_stone(StoneKind::Amber));
        assert!(!t.try_add_stone(StoneKind::Amethyst));
        assert!(!t.try_add_stone(StoneKind::Jade)); // already present
        assert_eq!(t.stone_count(), 3);
    }

    #[test]
    fn generation_places_food_and_all_six_stones() {
        let world = World::generate(20, 20, 42);
        world.assert_invariants();
        assert!(world.count_food() > 0, "food must exist (AQ27)");
        for kind in StoneKind::ALL {
            assert!(
                world.count_stone(kind) > 0,
                "stone {kind} must exist (AQ28)"
            );
        }
    }

    #[test]
    fn generation_obeys_per_tile_rules() {
        let world = World::generate(15, 15, 7);
        for tile in &world.tiles {
            assert!(tile.stone_count() <= 3);
            // Duplicate type impossible by bool array.
        }
    }

    #[test]
    fn stones_are_not_all_on_one_tile() {
        let world = World::generate(12, 12, 99);
        let total = world.count_all_stones();
        assert!(total > 3, "expected a spread of stones, got {total}");
        let tiles_with_stones = world.tiles.iter().filter(|t| t.stone_count() > 0).count();
        assert!(
            tiles_with_stones > 1,
            "stones must not all sit on one square"
        );
    }

    #[test]
    fn same_seed_is_deterministic() {
        let a = World::generate(8, 8, 12345);
        let b = World::generate(8, 8, 12345);
        assert_eq!(a.tiles, b.tiles);
        assert_eq!(a.summary_line(), b.summary_line());
    }

    #[test]
    fn different_seeds_usually_differ() {
        let a = World::generate(10, 10, 1);
        let b = World::generate(10, 10, 2);
        assert_ne!(a.tiles, b.tiles);
    }

    #[test]
    fn densities_are_near_targets_on_large_map() {
        let world = World::generate(40, 40, 2026);
        let n = world.tile_count() as f64;
        let food_ratio = world.count_food() as f64 / n;
        assert!(
            (food_ratio - FOOD_DENSITY).abs() < 0.05,
            "food density {food_ratio} far from {FOOD_DENSITY}"
        );
        for kind in StoneKind::ALL {
            let target = STONE_DENSITIES[kind as usize];
            let ratio = world.count_stone(kind) as f64 / n;
            // Cap from max-3-kinds can clip denser stones slightly on busy maps.
            assert!(
                ratio <= target + 0.02,
                "{kind} density {ratio} above target {target}"
            );
            assert!(
                ratio >= target * 0.5,
                "{kind} density {ratio} far below target {target}"
            );
        }
    }

    #[test]
    fn respawn_tick_refills_after_clear() {
        let mut world = World::generate(10, 10, 5);
        // Strip all food.
        for tile in &mut world.tiles {
            tile.food = false;
        }
        assert_eq!(world.count_food(), 0);
        let mut rng = SeededRng::new(5);
        world.respawn_tick(&mut rng);
        assert!(world.count_food() > 0, "respawn should place some food");
        world.assert_invariants();
    }

    #[test]
    fn summary_line_lists_resources() {
        let world = World::generate(5, 5, 1);
        let s = world.summary_line();
        assert!(s.contains("food="));
        assert!(s.contains("jade="));
        assert!(s.contains("ammolite="));
    }
}
