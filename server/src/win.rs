//! Win detection (S15 / RQ02).
//!
//! The game ends when at least [`WIN_PLAYERS`] living members of one team are at
//! level [`WIN_LEVEL`] (8). There is no AI-client protocol line for victory in
//! the subject command table; the server logs the winner and stops the event
//! loop.

use crate::player::PlayerSet;

/// Highest player level / win threshold (subject).
pub const WIN_LEVEL: u8 = 8;

/// Members of one team required at [`WIN_LEVEL`] to end the game.
pub const WIN_PLAYERS: usize = 6;

/// If any team has ≥[`WIN_PLAYERS`] living players at level ≥[`WIN_LEVEL`],
/// return that team name.
pub fn winning_team(players: &PlayerSet) -> Option<&str> {
    for p in players.iter() {
        if count_at_level(players, &p.team, WIN_LEVEL) >= WIN_PLAYERS {
            return Some(p.team.as_str());
        }
    }
    None
}

/// How many living players on `team` are at least `level`.
pub fn count_at_level(players: &PlayerSet, team: &str, level: u8) -> usize {
    players
        .iter()
        .filter(|p| p.team == team && p.level >= level)
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player::PlayerSet;
    use crate::world::{SeededRng, World};

    fn add(players: &mut PlayerSet, team: &str, level: u8) -> u32 {
        let world = World::empty(4, 4);
        let mut rng = SeededRng::new(1);
        let id = players.spawn(team, &world, &mut rng);
        players.get_mut(id).unwrap().level = level;
        id
    }

    #[test]
    fn constants_match_subject() {
        assert_eq!(WIN_LEVEL, 8);
        assert_eq!(WIN_PLAYERS, 6);
    }

    #[test]
    fn no_winner_below_threshold() {
        let mut players = PlayerSet::default();
        for _ in 0..5 {
            add(&mut players, "alpha", 8);
        }
        add(&mut players, "alpha", 7);
        assert!(winning_team(&players).is_none());
    }

    #[test]
    fn six_at_level_eight_wins() {
        let mut players = PlayerSet::default();
        for _ in 0..6 {
            add(&mut players, "alpha", 8);
        }
        add(&mut players, "beta", 8);
        assert_eq!(winning_team(&players), Some("alpha"));
    }

    #[test]
    fn other_team_does_not_count() {
        let mut players = PlayerSet::default();
        for _ in 0..5 {
            add(&mut players, "alpha", 8);
        }
        for _ in 0..5 {
            add(&mut players, "beta", 8);
        }
        assert!(winning_team(&players).is_none());
    }

    #[test]
    fn first_team_to_six_wins() {
        let mut players = PlayerSet::default();
        for _ in 0..6 {
            add(&mut players, "beta", 8);
        }
        for _ in 0..6 {
            add(&mut players, "alpha", 8);
        }
        // Iteration order is hash-map based; either team is a valid winner.
        let w = winning_team(&players).unwrap();
        assert!(w == "alpha" || w == "beta");
        assert_eq!(count_at_level(&players, w, 8), 6);
    }
}
