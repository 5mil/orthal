//! After quote_raised hits graduate_quote, inventory and vault become PoolLp notes.

use super::action::{ActionBundle, CompactAction, CompactOutput, CompactSpend};
use super::asset::{normalize_symbol, ticker_id, ORTH};
use super::auth::{rerand, RangeProof};
use super::commitment::{blinding_from_seed, ValueCommitment};
use super::keys::SpendKey;
use super::launch::LaunchSet;
use super::pred::{PredHeader, PredWitness, Predicate};
use super::proof::{commit_with_asset, BindingSig, ImageOr, NoteProof};
use super::ticker::{inventory_blind, inventory_cm, CurveSpec};
use super::trade::{vault_blind, vault_cm};
use curve25519_dalek::constants::RISTRETTO_BASEPOINT_POINT;
use curve25519_dalek::scalar::Scalar;

fn eph(div: [u8; 16], pk: &[u8; 32]) -> [u8; 32] {
    let sk = blinding_from_seed(&[b"eph-g".as_ref(), &div, pk].concat());
    (sk * RISTRETTO_BASEPOINT_POINT).compress().to_bytes()
}

fn lp_pred(spec: &CurveSpec) -> Predicate {
    Predicate::PoolLp { pool: spec.asset, asset_a: spec.asset, asset_b: ORTH }
}

pub fn graduate_spend(
    keeper: &SpendKey, set: &LaunchSet, spec: &CurveSpec, symbol: &str, height: u64,
) -> Result<ActionBundle, &'static str> {
    if !spec.graduated() { return Err("not graduated"); }
    let sym = normalize_symbol(symbol)?.to_string();
    if ticker_id(&sym)? != spec.asset { return Err("symbol/asset mismatch"); }
    let pred = spec.as_pred();
    let header = PredHeader::from_pred(&pred);
    let inv_cm = inventory_cm(spec);
    let v_cm = vault_cm(spec);
    if !set.contains(inv_cm) || !set.contains(v_cm) { return Err("inventory/vault not live"); }
    let leaves = set.live_leaves_for(header.id, header.commit);
    let inv_i = leaves.iter().position(|l| *l == inv_cm).ok_or("inv slice")?;
    let v_i = leaves.iter().position(|l| *l == v_cm).ok_or("vault slice")?;
    let inv_b = inventory_blind(spec);
    let v_b = vault_blind(spec);
    let mut ctx = Vec::new();
    ctx.extend_from_slice(&set.window_root());
    ctx.extend_from_slice(&height.to_le_bytes());
    fn spend_one(
        sk: &SpendKey, leaves: &[[u8; 32]], idx: usize, cm: [u8; 32], value: u64, blind: &Scalar,
        pred: &Predicate, header: &PredHeader, ctx: &[u8], dummy: &BindingSig,
    ) -> Result<(CompactSpend, Scalar), &'static str> {
        let expected = commit_with_asset(value, blind, &ORTH);
        if expected.commitment != cm { return Err("opening"); }
        let delta = blinding_from_seed(&[b"rerand".as_ref(), &sk.sk.to_bytes(), &cm].concat());
        let c_prime = rerand(&expected, &delta);
        let image = sk.key_image(&cm);
        let image_or = ImageOr::prove(leaves, idx, sk, &delta, &image, &c_prime, ctx)?;
        Ok((CompactSpend {
            spend_tag: image, rerand: c_prime,
            proof: Some(NoteProof {
                image: image_or,
                range_in: RangeProof::prove(value, &(*blind + delta)),
                range_outs: vec![],
                range_fee: RangeProof::prove(0, &Scalar::ZERO),
                binding: dummy.clone(),
            }),
            pred: header.clone(), pred_body: Some(pred.clone()),
            pred_witness: PredWitness { keys: vec![], fill: b"graduate".to_vec() },
        }, delta))
    }
    let dummy = BindingSig::sign(&blinding_from_seed(b"dummy-bind-nonzero"), b"tmp")?;
    let (s0, d0) = spend_one(keeper, &leaves, inv_i, inv_cm, spec.cap_remaining, &inv_b, &pred, &header, &ctx, &dummy)?;
    let (s1, d1) = spend_one(keeper, &leaves, v_i, v_cm, spec.quote_raised, &v_b, &pred, &header, &ctx, &dummy)?;
    let lp = lp_pred(spec);
    let lp_h = PredHeader::from_pred(&lp);
    let tok_r = blinding_from_seed(&[b"lp-tok".as_ref(), &spec.asset].concat());
    let q_r = blinding_from_seed(&[b"lp-q".as_ref(), &spec.asset].concat());
    let o_tok = CompactOutput {
        dest: keeper.pk(), eph_pk: eph([9u8; 16], &keeper.pk().bytes), diversifier: [9u8; 16],
        value_commitment: commit_with_asset(spec.cap_remaining, &tok_r, &ORTH),
        pred: lp_h.clone(), asset: spec.asset, symbol: sym,
    };
    let o_q = CompactOutput {
        dest: keeper.pk(), eph_pk: eph([10u8; 16], &keeper.pk().bytes), diversifier: [10u8; 16],
        value_commitment: commit_with_asset(spec.quote_raised, &q_r, &ORTH),
        pred: lp_h, asset: ORTH, symbol: String::new(),
    };
    let outs = vec![o_tok.value_commitment.clone(), o_q.value_commitment.clone()];
    let fee = ValueCommitment::identity();
    let r_bind = (inv_b + d0) + (v_b + d1) - tok_r - q_r;
    let mut tr = Vec::new();
    tr.extend_from_slice(&s0.spend_tag);
    tr.extend_from_slice(&s1.spend_tag);
    for o in &outs { tr.extend_from_slice(&o.commitment); }
    tr.extend_from_slice(&fee.commitment);
    tr.extend_from_slice(&set.window_root());
    tr.extend_from_slice(&height.to_le_bytes());
    let binding = BindingSig::sign(&r_bind, &tr)?;
    let range_outs = vec![RangeProof::prove(spec.cap_remaining, &tok_r), RangeProof::prove(spec.quote_raised, &q_r)];
    let mut spends = vec![s0, s1];
    for s in spends.iter_mut() {
        if let Some(p) = s.proof.as_mut() {
            p.range_outs = range_outs.clone();
            p.range_fee = RangeProof::prove(0, &Scalar::ZERO);
            p.binding = binding.clone();
        }
    }
    let bundle = ActionBundle {
        version: 7,
        actions: vec![
            CompactAction { spend: Some(spends.remove(0)), output: Some(o_tok) },
            CompactAction { spend: Some(spends.remove(0)), output: Some(o_q) },
        ],
        fee_commitment: fee, binding: Some(binding), emission: None,
        intents: vec![], fills: vec![], exec: None,
    };
    if !bundle.verify_conservation() { return Err("conservation failed"); }
    Ok(bundle)
}
