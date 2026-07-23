//! Zappy game server library.
//!
//! Scaffold only (A03). Game logic lands in later server tickets.

/// Placeholder so `cargo test` / `clippy` have a unit to exercise.
#[must_use]
pub fn scaffold_ok() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scaffold_ok_is_true() {
        assert!(scaffold_ok());
    }
}
