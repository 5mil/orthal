use crate::params::CHAIN_PARAMS;

/// Bit-count difficulty. Work is ~2^d, so a 2× fast window adds one bit.
/// `actual` is clamped to [target/4, 4*target] before the step.
pub fn retarget(current_difficulty: u32, actual_timespan: u64, target_timespan: u64) -> u32 {
    if target_timespan == 0 { return current_difficulty; }
    let floor = CHAIN_PARAMS.initial_difficulty;
    let clamped = actual_timespan.max(target_timespan / 4).min(target_timespan.saturating_mul(4));
    let mut d = current_difficulty;
    if clamped < target_timespan {
        let mut span = clamped.max(1);
        while span.saturating_mul(2) <= target_timespan && d < 240 {
            span = span.saturating_mul(2);
            d = d.saturating_add(1);
        }
    } else if clamped > target_timespan {
        let mut span = target_timespan.max(1);
        while span.saturating_mul(2) <= clamped && d > floor {
            span = span.saturating_mul(2);
            d = d.saturating_sub(1);
        }
    }
    d.max(floor).min(240)
}

pub fn pow_target_timespan() -> u64 {
    CHAIN_PARAMS.difficulty_adjustment_window * CHAIN_PARAMS.pow_target_block_time
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_difficulty_increases_when_fast() {
        let target = pow_target_timespan();
        assert!(retarget(20, target / 2, target) > 20);
        assert_eq!(retarget(20, target / 4, target), 22);
    }
    #[test]
    fn test_difficulty_decreases_when_slow() {
        let target = pow_target_timespan();
        assert!(retarget(20, target * 2, target) < 20);
        assert_eq!(retarget(20, target * 4, target), 18);
    }
    #[test]
    fn floor_is_initial() {
        let target = pow_target_timespan();
        assert_eq!(retarget(CHAIN_PARAMS.initial_difficulty, target * 4, target), CHAIN_PARAMS.initial_difficulty);
    }
}
