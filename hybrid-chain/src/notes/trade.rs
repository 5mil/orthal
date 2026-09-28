//! Multi-input curve fills. Vault holds raised ORTH so a sell cannot mint quote.

use super::action::{ActionBundle, CompactAction, CompactOutput, CompactSpend};
use super::asset::{normalize_symbol, ticker_id, ORTH};
use super::auth::{rerand, RangeProof};
use super::commitment::{blinding_from_seed, ValueCommitment};
use super::keys::SpendKey;
use super::launch::LaunchSet;
use super::pred::{PredHeader, PredWitness, Predicate};
use super::proof::{commit_with_asset, BindingSig, ImageOr, NoteProof};
use super::ticker::{inventory_blind, inventory_cm, CurveSpec};
use curve25519_dalek::constants::RISTRETTO_BASEPOINT_POINT;
use curve25519_dalek::scalar::Scalar;

pub fn vault_blind(spec: &CurveSpec) -> Scalar {
    blinding_from_seed(&[b"vault-r".as_ref(), &spec.asset, &spec.quote_raised.to_le_bytes()].concat())
}

pub fn vault_cm(spec: &CurveSpec) -> [u8; 32] {
    commit_with_asset(spec.quote_raised, &vault_blind(spec), &ORTH).commitment
}

fn eph(div: [u8; 16], pk: &[u8; 32]) -> [u8; 32] {
    let sk = blinding_from_seed(&[b"eph-v".as_ref(), &div, pk].concat());
    (sk * RISTRETTO_BASEPOINT_POINT).compress().to_bytes()
}

fn out(dest: &SpendKey, value: u64, blind: &Scalar, asset: [u8; 32], symbol: String, pred: PredHeader, div: [u8; 16]) -> CompactOutput {
    CompactOutput {
        dest: dest.pk(), eph_pk: eph(div, &dest.pk().bytes), diversifier: div,
        value_commitment: commit_with_asset(value, blind, &ORTH), pred, asset, symbol,
    }
}

pub fn genesis_vault(keeper: &SpendKey, spec: &CurveSpec) -> CompactOutput {
    out(keeper, spec.quote_raised, &vault_blind(spec), ORTH, String::new(), PredHeader::from_pred(&spec.as_pred()), [8u8; 16])
}

struct Leg {
    sk: SpendKey,
    cm: [u8; 32],
    value: u64,
    blind: Scalar,
    pred: Predicate,
}

fn prove_leg(
    leg: &Leg, set: &LaunchSet, height: u64, ctx: &[u8],
    outs: &[ValueCommitment], fee: &ValueCommitment, dummy_bind: &BindingSig,
) -> Result<(CompactSpend, Scalar, ValueCommitment), &'static str> {
    if !set.contains(leg.cm) { return Err("leg not live"); }
    let header = PredHeader::from_pred(&leg.pred);
    let leaves = set.live_leaves_for(header.id, header.commit);
    let idx = leaves.iter().position(|l| *l == leg.cm).ok_or("leg not in slice")?;
    let expected = commit_with_asset(leg.value, &leg.blind, &ORTH);
    if expected.commitment != leg.cm { return Err("leg opening"); }
    let delta = blinding_from_seed(&[b"rerand".as_ref(), &leg.sk.sk.to_bytes(), &leg.cm].concat());
    let c_prime = rerand(&expected, &delta);
    let image = leg.sk.key_image(&leg.cm);
    let image_or = ImageOr::prove(&leaves, idx, &leg.sk, &delta, &image, &c_prime, ctx)?;
    let ranges: Vec<RangeProof> = outs.iter().map(|_| RangeProof::prove(0, &Scalar::ZERO)).collect();
    // range_outs on each leg are checked against real outputs; fill properly at caller.
    let _ = ranges;
    let proof = NoteProof {
        image: image_or,
        range_in: RangeProof::prove(leg.value, &(leg.blind + delta)),
        range_outs: vec![],
        range_fee: RangeProof::prove(0, &Scalar::ZERO),
        binding: dummy_bind.clone(),
    };
    let _ = fee;
    Ok((CompactSpend {
        spend_tag: image, rerand: c_prime, proof: Some(proof),
        pred: header, pred_body: if header.is_default() { None } else { Some(leg.pred.clone()) },
        pred_witness: PredWitness::none(),
    }, delta, c_prime))
}

fn finish_proofs(
    spends: &mut [CompactSpend], values_blinds: &[(u64, Scalar)],
    out_vals: &[(u64, Scalar)], fee: &ValueCommitment,
) {
    let range_outs: Vec<RangeProof> = out_vals.iter().map(|(v, r)| RangeProof::prove(*v, r)).collect();
    let range_fee = RangeProof::prove(0, &Scalar::ZERO);
    for (s, (v, r)) in spends.iter_mut().zip(values_blinds.iter()) {
        if let Some(p) = s.proof.as_mut() {
            p.range_outs = range_outs.clone();
            p.range_fee = range_fee.clone();
            let _ = v; let _ = r;
        }
    }
    let _ = fee;
}

/// Spend inventory + vault + seller tokens. Pay quote ORTH from the vault.
pub fn sell_spend(
    keeper: &SpendKey, seller: &SpendKey, set: &LaunchSet, spec: &CurveSpec,
    symbol: &str, tokens_in: u64, height: u64,
    seller_token_cm: [u8; 32], seller_token_blind: Scalar,
) -> Result<(ActionBundle, CurveSpec, u64), &'static str> {
    let (next, quote) = spec.after_sell(tokens_in)?;
    let sym = normalize_symbol(symbol)?.to_string();
    if ticker_id(&sym)? != spec.asset { return Err("symbol/asset mismatch"); }
    if spec.quote_raised < quote { return Err("vault dry"); }
    let inv = Leg { sk: keeper.clone(), cm: inventory_cm(spec), value: spec.cap_remaining, blind: inventory_blind(spec), pred: spec.as_pred() };
    let vault = Leg { sk: keeper.clone(), cm: vault_cm(spec), value: spec.quote_raised, blind: vault_blind(spec), pred: spec.as_pred() };
    let tok = Leg { sk: seller.clone(), cm: seller_token_cm, value: tokens_in, blind: seller_token_blind, pred: Predicate::Pk };
    let mut ctx = Vec::new();
    ctx.extend_from_slice(&set.window_root());
    ctx.extend_from_slice(&height.to_le_bytes());
    let inv_r = inventory_blind(&next);
    let v_r = vault_blind(&next);
    let q_r = blinding_from_seed(&[b"sell-r".as_ref(), &spec.asset, &quote.to_le_bytes()].concat());
    let o_inv = out(keeper, next.cap_remaining, &inv_r, spec.asset, sym, PredHeader::from_pred(&next.as_pred()), [5u8; 16]);
    let o_vault = out(keeper, next.quote_raised, &v_r, ORTH, String::new(), PredHeader::from_pred(&next.as_pred()), [8u8; 16]);
    let o_pay = out(seller, quote, &q_r, ORTH, String::new(), PredHeader::pk(), [6u8; 16]);
    let outs = vec![o_inv.value_commitment.clone(), o_vault.value_commitment.clone(), o_pay.value_commitment.clone()];
    let fee = ValueCommitment::identity();
    let dummy = BindingSig::sign(&blinding_from_seed(b"dummy-bind-nonzero"), b"tmp")?;
    let (s0, d0, _) = prove_leg(&inv, set, height, &ctx, &outs, &fee, &dummy)?;
    let (s1, d1, _) = prove_leg(&vault, set, height, &ctx, &outs, &fee, &dummy)?;
    let (s2, d2, _) = prove_leg(&tok, set, height, &ctx, &outs, &fee, &dummy)?;
    let r_bind = (inv.blind + d0) + (vault.blind + d1) + (tok.blind + d2) - inv_r - v_r - q_r;
    let mut transcript = Vec::new();
    transcript.extend_from_slice(&s0.spend_tag);
    transcript.extend_from_slice(&s1.spend_tag);
    transcript.extend_from_slice(&s2.spend_tag);
    for o in &outs { transcript.extend_from_slice(&o.commitment); }
    transcript.extend_from_slice(&fee.commitment);
    transcript.extend_from_slice(&set.window_root());
    transcript.extend_from_slice(&height.to_le_bytes());
    let binding = BindingSig::sign(&r_bind, &transcript)?;
    let mut spends = vec![s0, s1, s2];
    finish_proofs(&mut spends, &[(inv.value, inv.blind + d0), (vault.value, vault.blind + d1), (tok.value, tok.blind + d2)], &[(next.cap_remaining, inv_r), (next.quote_raised, v_r), (quote, q_r)], &fee);
    for s in spends.iter_mut() {
        if let Some(p) = s.proof.as_mut() { p.binding = binding.clone(); }
    }
    let bundle = ActionBundle {
        version: 7,
        actions: vec![
            CompactAction { spend: Some(spends.remove(0)), output: Some(o_inv) },
            CompactAction { spend: Some(spends.remove(0)), output: Some(o_vault) },
            CompactAction { spend: Some(spends.remove(0)), output: Some(o_pay) },
        ],
        fee_commitment: fee, binding: Some(binding), emission: None,
        intents: vec![], fills: vec![], exec: None,
    };
    if !bundle.verify_conservation() { return Err("conservation failed"); }
    Ok((bundle, next, quote))
}
