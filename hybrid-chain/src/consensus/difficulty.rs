//! ASERT on a 256-bit PoW target. PoS does not use this module.

use crate::params::CHAIN_PARAMS;

pub type Target = [u8; 32];

const LN2_NUM: u64 = 45_426;
const LN2_DEN: u64 = 65_536;

pub fn target_from_bits(bits: u32) -> Target {
    let d = bits.min(255);
    let mut t = [0xffu8; 32];
    let full = (d / 8) as usize;
    for i in 0..full { t[i] = 0; }
    let rem = d % 8;
    if full < 32 && rem > 0 {
        t[full] = 0xff >> rem;
    }
    if d == 256 { t = [0u8; 32]; }
    t
}

pub fn bits_of(target: &Target) -> u32 {
    let mut bits = 0u32;
    for b in target {
        if *b == 0 { bits += 8; continue; }
        bits += b.leading_zeros();
        break;
    }
    bits.min(256)
}

pub fn meets_target(hash: &[u8; 32], target: &Target) -> bool {
    hash.as_slice() <= target.as_slice()
}

pub fn genesis_target() -> Target {
    target_from_bits(CHAIN_PARAMS.initial_difficulty)
}

fn clamp_target(t: Target) -> Target {
    let min_t = target_from_bits(CHAIN_PARAMS.max_pow_bits);
    let max_t = target_from_bits(CHAIN_PARAMS.min_pow_bits);
    if t.as_slice() < min_t.as_slice() { return min_t; }
    if t.as_slice() > max_t.as_slice() { return max_t; }
    t
}

fn from_be(t: &Target) -> [u64; 4] {
    [
        u64::from_be_bytes(t[24..32].try_into().unwrap()),
        u64::from_be_bytes(t[16..24].try_into().unwrap()),
        u64::from_be_bytes(t[8..16].try_into().unwrap()),
        u64::from_be_bytes(t[0..8].try_into().unwrap()),
    ]
}

fn to_be(limbs: [u64; 4]) -> Target {
    let mut t = [0u8; 32];
    t[0..8].copy_from_slice(&limbs[3].to_be_bytes());
    t[8..16].copy_from_slice(&limbs[2].to_be_bytes());
    t[16..24].copy_from_slice(&limbs[1].to_be_bytes());
    t[24..32].copy_from_slice(&limbs[0].to_be_bytes());
    t
}

fn mul_u64(limbs: [u64; 4], n: u64) -> [u64; 5] {
    let mut out = [0u64; 5];
    let mut carry = 0u128;
    for i in 0..4 {
        let v = limbs[i] as u128 * n as u128 + carry;
        out[i] = v as u64;
        carry = v >> 64;
    }
    out[4] = carry as u64;
    out
}

fn div_u64(mut limbs: [u64; 5], d: u64) -> [u64; 4] {
    assert!(d != 0);
    let mut rem = 0u128;
    for i in (0..5).rev() {
        let cur = (rem << 64) | limbs[i] as u128;
        limbs[i] = (cur / d as u128) as u64;
        rem = cur % d as u128;
    }
    [limbs[0], limbs[1], limbs[2], limbs[3]]
}

fn mul_div(t: Target, num: u64, den: u64) -> Target {
    if den == 0 || num == 0 { return if num == 0 { [0u8; 32] } else { t }; }
    to_be(div_u64(mul_u64(from_be(&t), num), den))
}

fn shl_bits(t: Target, bits: u32) -> Target {
    if bits == 0 { return t; }
    if bits >= 256 { return [0xff; 32]; }
    let mut limbs = from_be(&t);
    let word = (bits / 64) as usize;
    let rem = bits % 64;
    if word > 0 {
        for i in (word..4).rev() { limbs[i] = limbs[i - word]; }
        for i in 0..word { limbs[i] = 0; }
    }
    if rem > 0 {
        let mut carry = 0u64;
        for i in 0..4 {
            let new_carry = limbs[i] >> (64 - rem);
            limbs[i] = (limbs[i] << rem) | carry;
            carry = new_carry;
        }
    }
    to_be(limbs)
}

fn shr_bits(t: Target, bits: u32) -> Target {
    if bits == 0 { return t; }
    if bits >= 256 { return [0u8; 32]; }
    let mut limbs = from_be(&t);
    let word = (bits / 64) as usize;
    let rem = bits % 64;
    if word > 0 {
        for i in 0..(4 - word) { limbs[i] = limbs[i + word]; }
        for i in (4 - word)..4 { limbs[i] = 0; }
    }
    if rem > 0 {
        let mut carry = 0u64;
        for i in (0..4).rev() {
            let new_carry = limbs[i] << (64 - rem);
            limbs[i] = (limbs[i] >> rem) | carry;
            carry = new_carry;
        }
    }
    to_be(limbs)
}

/// `target * 2^(time_error / half_life)` relative to an anchor.
/// `time_diff` is wall seconds since the anchor PoW header.
/// `pow_intervals` is PoW blocks since that anchor (not mixed height).
pub fn asert(anchor: Target, time_diff: i64, pow_intervals: u64) -> Target {
    let tau = CHAIN_PARAMS.pow_target_block_time as i64;
    let half = CHAIN_PARAMS.asert_half_life;
    if half <= 0 { return clamp_target(anchor); }
    let ideal = (pow_intervals as i64).saturating_mul(tau);
    let mut exponent = time_diff.saturating_sub(ideal);
    let cap = half.saturating_mul(16);
    exponent = exponent.clamp(-cap, cap);
    let shifts = exponent / half;
    let frac = exponent % half;
    let mut t = anchor;
    if shifts > 0 {
        t = shl_bits(t, shifts as u32);
    } else if shifts < 0 {
        t = shr_bits(t, (-shifts) as u32);
    }
    if frac != 0 {
        let mag = frac.unsigned_abs();
        let add = mag.saturating_mul(LN2_NUM) / LN2_DEN;
        if frac > 0 {
            t = mul_div(t, half as u64 + add, half as u64);
        } else {
            t = mul_div(t, half as u64, half as u64 + add.max(1));
        }
    }
    clamp_target(t)
}

/// Approximate work = 2^{leading_zero_bits(target)}.
pub fn work_from_target(target: &Target) -> [u8; 32] {
    let bits = bits_of(target).min(255);
    let mut w = [0u8; 32];
    let byte = 31 - (bits as usize / 8);
    let rem = bits % 8;
    w[byte] = 1u8 << rem;
    w
}

pub fn add_work(acc: [u8; 32], one: [u8; 32]) -> [u8; 32] {
    let mut out = [0u8; 32];
    let mut carry = 0u16;
    for i in (0..32).rev() {
        let s = acc[i] as u16 + one[i] as u16 + carry;
        out[i] = s as u8;
        carry = s >> 8;
    }
    out
}

pub fn work_cmp(a: &[u8; 32], b: &[u8; 32]) -> std::cmp::Ordering {
    a.as_slice().cmp(b.as_slice())
}

/// Kept for tests that still speak windows. Not used on the live path.
pub fn retarget(current_difficulty: u32, actual_timespan: u64, target_timespan: u64) -> u32 {
    if target_timespan == 0 { return current_difficulty; }
    let anchor = target_from_bits(current_difficulty);
    let next = asert(anchor, actual_timespan as i64, 1);
    bits_of(&next).max(CHAIN_PARAMS.min_pow_bits).min(CHAIN_PARAMS.max_pow_bits)
}

pub fn pow_target_timespan() -> u64 {
    CHAIN_PARAMS.difficulty_adjustment_window * CHAIN_PARAMS.pow_target_block_time
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fast_blocks_tighten_target() {
        let anchor = genesis_target();
        let next = asert(anchor, 60, 1);
        assert!(next.as_slice() < anchor.as_slice(), "fast window must shrink target");
    }
    #[test]
    fn slow_blocks_ease_target() {
        let anchor = genesis_target();
        let next = asert(anchor, 240, 1);
        assert!(next.as_slice() > anchor.as_slice(), "slow window must grow target");
    }
    #[test]
    fn on_time_stays_near_anchor() {
        let anchor = genesis_target();
        let next = asert(anchor, 120, 1);
        assert_eq!(bits_of(&next), bits_of(&anchor));
    }
    #[test]
    fn target_from_bits_matches_old_pow() {
        let t = target_from_bits(12);
        assert_eq!(t[0], 0);
        assert_eq!(t[1], 0x0f);
        assert!(meets_target(&[0u8; 32], &t));
        let mut high = [0u8; 32];
        high[1] = 0x10;
        assert!(!meets_target(&high, &t));
    }
    #[test]
    fn work_accumulates() {
        let w = work_from_target(&genesis_target());
        let sum = add_work(w, w);
        assert!(work_cmp(&sum, &w) == std::cmp::Ordering::Greater);
    }
}
