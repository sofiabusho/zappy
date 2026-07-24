//! Game time unit helpers (S06 / RQ10).
//!
//! A time unit lasts `1/t` seconds. An action that costs `cost` time units
//! therefore lasts `cost/t` seconds. Larger `t` → faster game (default 100).

use std::time::Duration;

/// Convert an action cost in time units into a wall-clock [`Duration`] for divider `t`.
///
/// # Panics
/// Panics if `t == 0` (CLI validation already rejects that).
pub fn action_duration(t: u32, cost_tu: u32) -> Duration {
    assert!(t > 0, "t must be > 0");
    if cost_tu == 0 {
        return Duration::ZERO;
    }
    // Prefer integer nanos when possible to avoid float drift on common values.
    // duration = cost / t seconds = (cost * 1_000_000_000) / t nanoseconds.
    let cost = u128::from(cost_tu);
    let divider = u128::from(t);
    let nanos = (cost * 1_000_000_000) / divider;
    Duration::from_nanos(nanos as u64)
}

/// Duration of one time unit at divider `t`.
pub fn time_unit_duration(t: u32) -> Duration {
    action_duration(t, 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_t_advance_is_70ms() {
        // t=100, cost=7 → 7/100 s = 70 ms (RQ10 example family).
        assert_eq!(action_duration(100, 7), Duration::from_millis(70));
    }

    #[test]
    fn t_equals_1_advance_is_7_seconds() {
        assert_eq!(action_duration(1, 7), Duration::from_secs(7));
    }

    #[test]
    fn zero_cost_is_instant() {
        assert_eq!(action_duration(100, 0), Duration::ZERO);
    }

    #[test]
    fn one_time_unit_at_t100() {
        assert_eq!(time_unit_duration(100), Duration::from_millis(10));
    }
}
