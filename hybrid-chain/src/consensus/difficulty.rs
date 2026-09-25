use crate::params::CHAIN_PARAMS;

pub fn retarget(current_difficulty: u32, actual_timespan: u64, target_timespan: u64) -> u32 {
    let clamped_actual = actual_timespan.max(target_timespan / 4).min(target_timespan * 4);
    let ratio = (clamped_actual * 1000) / target_timespan;
    if ratio < 1000 {
        (current_difficulty + 1).min(240)
    } else if ratio > 1000 {
        current_difficulty.saturating_sub(1).max(CHAIN_PARAMS.initial_difficulty)
    } else {
        current_difficulty
    }
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
    }
    #[test]
    fn test_difficulty_decreases_when_slow() {
        let target = pow_target_timespan();
        assert!(retarget(20, target * 2, target) < 20);
    }
}
