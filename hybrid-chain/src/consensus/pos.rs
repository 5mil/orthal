use crate::params::CHAIN_PARAMS;

pub fn calculate_coin_age(coins: u64, seconds_held: u64) -> u64 {
    coins.saturating_mul(seconds_held.min(CHAIN_PARAMS.pos_coin_age_max))
}

pub fn stake_weight(coins: u64, seconds_held: u64) -> u64 {
    if seconds_held < CHAIN_PARAMS.pos_coin_age_min { return 0; }
    if coins < CHAIN_PARAMS.min_stake { return 0; }
    calculate_coin_age(coins, seconds_held)
}

pub fn pos_reward(coins: u64, seconds_held: u64) -> u64 {
    let days_held = seconds_held.min(CHAIN_PARAMS.pos_coin_age_max) as f64 / 86400.0;
    ((coins as f64) * CHAIN_PARAMS.pos_annual_rate * (days_held / 365.0)) as u64
}

pub fn validate_stake(coins: u64, seconds_held: u64) -> Result<(), &'static str> {
    if coins < CHAIN_PARAMS.min_stake { return Err("Insufficient stake: below minimum"); }
    if seconds_held < CHAIN_PARAMS.pos_coin_age_min { return Err("Coin age too low: coins must be held at least 1 day"); }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_coin_age_capped() {
        let max_age = CHAIN_PARAMS.pos_coin_age_max;
        assert_eq!(calculate_coin_age(1000, max_age + 1000), calculate_coin_age(1000, max_age));
    }
    #[test]
    fn test_stake_weight_immature() {
        assert_eq!(stake_weight(10_000_0000_0000, 3600), 0);
    }
}
