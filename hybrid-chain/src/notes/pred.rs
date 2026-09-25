//! Listed predicates on notes. Unknown pred ids fail closed.

use super::keys::SpendPk;
use crate::consensus::pow::sha256d;
use serde::{Deserialize, Serialize};

pub const PRED_PK: [u8; 32] = [0u8; 32];

fn tag(label: &[u8]) -> [u8; 32] {
    sha256d(&[b"orthal-pred-id".as_ref(), label].concat())
}

pub fn id_pk_n() -> [u8; 32] { tag(b"pk-n") }
pub fn id_after() -> [u8; 32] { tag(b"after") }
pub fn id_and() -> [u8; 32] { tag(b"and") }
pub fn id_or() -> [u8; 32] { tag(b"or") }
pub fn id_rate() -> [u8; 32] { tag(b"rate") }
pub fn id_swap() -> [u8; 32] { tag(b"swap") }
pub fn id_pool_lp() -> [u8; 32] { tag(b"pool-lp") }
pub fn id_curve() -> [u8; 32] { tag(b"curve") }
pub fn id_ticker() -> [u8; 32] { tag(b"ticker") }

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Predicate {
    Pk,
    PkN { dests: Vec<SpendPk>, n: u8 },
    After { height: u64 },
    And { left: Box<Predicate>, right: Box<Predicate> },
    Or { left: Box<Predicate>, right: Box<Predicate> },
    Rate { dest: SpendPk, max_per_window: u64, window: u64 },
    Swap { want_asset: [u8; 32], pay_asset: [u8; 32], max_in: u64, min_out: u64 },
    PoolLp { pool: [u8; 32], asset_a: [u8; 32], asset_b: [u8; 32] },
    Curve { asset: [u8; 32], cap: u64, virtual_quote: u64, graduate_quote: u64, start_height: u64, quote_raised: u64, cap_remaining: u64 },
    Ticker { asset: [u8; 32], symbol: String },
}

impl Predicate {
    pub fn id(&self) -> [u8; 32] {
        match self {
            Self::Pk => PRED_PK,
            Self::PkN { .. } => id_pk_n(),
            Self::After { .. } => id_after(),
            Self::And { .. } => id_and(),
            Self::Or { .. } => id_or(),
            Self::Rate { .. } => id_rate(),
            Self::Swap { .. } => id_swap(),
            Self::PoolLp { .. } => id_pool_lp(),
            Self::Curve { .. } => id_curve(),
            Self::Ticker { .. } => id_ticker(),
        }
    }
    pub fn commit(&self) -> [u8; 32] { sha256d(&bincode::serialize(self).unwrap_or_default()) }
    pub fn is_default(&self) -> bool { matches!(self, Self::Pk) }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PredWitness { pub keys: Vec<u8>, pub fill: Vec<u8> }
impl PredWitness { pub fn none() -> Self { Self::default() } }

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PredHeader { pub id: [u8; 32], pub commit: [u8; 32] }
impl PredHeader {
    pub fn pk() -> Self { Self { id: PRED_PK, commit: [0u8; 32] } }
    pub fn from_pred(p: &Predicate) -> Self {
        if p.is_default() { Self::pk() } else { Self { id: p.id(), commit: p.commit() } }
    }
    pub fn is_default(&self) -> bool { self.id == PRED_PK && self.commit == [0u8; 32] }
}

pub struct PredContext<'a> {
    pub height: u64,
    pub claimed: &'a Predicate,
    pub header: &'a PredHeader,
    pub witness: &'a PredWitness,
    pub out_dests: &'a [SpendPk],
}

pub fn verify_pred(ctx: PredContext<'_>) -> Result<(), &'static str> {
    if ctx.header.id != ctx.claimed.id() { return Err("pred id mismatch"); }
    if !ctx.claimed.is_default() && ctx.header.commit != ctx.claimed.commit() {
        return Err("pred commit mismatch");
    }
    eval(ctx.claimed, &ctx)
}

fn eval(p: &Predicate, ctx: &PredContext<'_>) -> Result<(), &'static str> {
    match p {
        Predicate::Pk => Ok(()),
        Predicate::PkN { dests, n } => {
            if *n as usize == 0 || dests.is_empty() { return Err("empty pk-n"); }
            let mut used = std::collections::BTreeSet::new();
            for i in &ctx.witness.keys {
                let i = *i as usize;
                if i >= dests.len() || !used.insert(i) { return Err("bad pk-n witness"); }
            }
            if used.len() < *n as usize { return Err("pk-n threshold"); }
            Ok(())
        }
        Predicate::After { height } => {
            if ctx.height >= *height { Ok(()) } else { Err("after: too early") }
        }
        Predicate::And { left, right } => { eval(left, ctx)?; eval(right, ctx) }
        Predicate::Or { left, right } => {
            if eval(left, ctx).is_ok() || eval(right, ctx).is_ok() { Ok(()) } else { Err("or: neither") }
        }
        Predicate::Rate { dest, .. } => {
            if ctx.out_dests.iter().any(|d| d == dest) { Ok(()) } else { Err("rate dest missing") }
        }
        Predicate::Swap { .. } => {
            if ctx.witness.fill.is_empty() { Err("swap needs fill") } else { Ok(()) }
        }
        Predicate::PoolLp { .. } => {
            if ctx.witness.fill.is_empty() { Err("lp needs fill") } else { Ok(()) }
        }
        Predicate::Ticker { asset, symbol } => {
            if *asset == [0u8; 32] || symbol.is_empty() { Err("ticker needs symbol") } else { Ok(()) }
        }
        Predicate::Curve { asset, start_height, .. } => {
            if ctx.height < *start_height { Err("curve closed") }
            else if *asset == [0u8; 32] { Err("curve on ORTH") }
            else { Ok(()) }
        }
    }
}

pub fn known_id(id: &[u8; 32]) -> bool {
    *id == PRED_PK || *id == id_pk_n() || *id == id_after() || *id == id_and()
        || *id == id_or() || *id == id_rate() || *id == id_swap() || *id == id_pool_lp()
        || *id == id_curve() || *id == id_ticker()
}
