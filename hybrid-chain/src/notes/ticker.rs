//! ORTH is the native ticker. Issued tickers are notes with a public asset id.

use super::action::{ActionBundle, CompactAction, CompactOutput, CompactSpend};
use super::asset::{normalize_symbol, ticker_id, ORTH};
use super::auth::{rerand, RangeProof};
use super::commitment::{blinding_from_seed, ValueCommitment};
use super::intent::{Intent, IntentFill};
use super::keys::{SpendKey, SpendPk};
use super::launch::LaunchSet;
use super::pred::{PredHeader, PredWitness, Predicate};
use super::proof::{commit_with_asset, BindingSig, ImageOr, NoteProof};
use curve25519_dalek::constants::RISTRETTO_BASEPOINT_POINT;
use curve25519_dalek::scalar::Scalar;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CurveSpec {
    pub asset: [u8; 32],
    pub cap: u64,
    pub virtual_quote: u64,
    pub graduate_quote: u64,
    pub start_height: u64,
    pub quote_raised: u64,
    pub cap_remaining: u64,
}

impl CurveSpec {
    pub fn new(asset: [u8; 32], cap: u64, virtual_quote: u64, graduate_quote: u64, start: u64) -> Self {
        Self { asset, cap, virtual_quote, graduate_quote, start_height: start, quote_raised: 0, cap_remaining: cap }
    }
    pub fn as_pred(&self) -> Predicate {
        Predicate::Curve {
            asset: self.asset, cap: self.cap, virtual_quote: self.virtual_quote,
            graduate_quote: self.graduate_quote, start_height: self.start_height,
            quote_raised: self.quote_raised, cap_remaining: self.cap_remaining,
        }
    }
    pub fn tokens_out(&self, quote_in: u64) -> Result<u64, &'static str> {
        if quote_in == 0 { return Err("zero buy"); }
        if self.graduated() { return Err("graduated"); }
        let den = self.virtual_quote.checked_add(self.quote_raised).and_then(|x| x.checked_add(quote_in)).ok_or("curve den")?;
        let num = (self.cap_remaining as u128).checked_mul(quote_in as u128).ok_or("curve num")?;
        let out = (num / den as u128) as u64;
        if out == 0 || out > self.cap_remaining { return Err("curve empty"); }
        Ok(out)
    }
    pub fn after_buy(&self, quote_in: u64) -> Result<(Self, u64), &'static str> {
        let got = self.tokens_out(quote_in)?;
        let mut n = self.clone();
        n.quote_raised = n.quote_raised.saturating_add(quote_in);
        n.cap_remaining = n.cap_remaining.saturating_sub(got);
        Ok((n, got))
    }
    pub fn quote_out(&self, tokens_in: u64) -> Result<u64, &'static str> {
        let sold = self.cap.saturating_sub(self.cap_remaining);
        if tokens_in == 0 || tokens_in > sold { return Err("sell exceeds sold"); }
        let q = self.virtual_quote.saturating_add(self.quote_raised);
        let den = self.cap_remaining.saturating_add(tokens_in);
        if den == 0 { return Err("sell den"); }
        let out = (q as u128 * tokens_in as u128 / den as u128) as u64;
        if out == 0 || out > self.quote_raised { return Err("sell empty"); }
        Ok(out)
    }
    pub fn after_sell(&self, tokens_in: u64) -> Result<(Self, u64), &'static str> {
        let got = self.quote_out(tokens_in)?;
        let mut n = self.clone();
        n.quote_raised = n.quote_raised.saturating_sub(got);
        n.cap_remaining = n.cap_remaining.saturating_add(tokens_in);
        Ok((n, got))
    }
    pub fn graduated(&self) -> bool { self.quote_raised >= self.graduate_quote }
}

fn eph(label: &[u8], div: &[u8; 16], pk: &[u8; 32]) -> [u8; 32] {
    let sk = blinding_from_seed(&[label, div.as_slice(), pk].concat());
    (sk * RISTRETTO_BASEPOINT_POINT).compress().to_bytes()
}

fn note(dest: SpendPk, value: u64, blind: &Scalar, asset: [u8; 32], symbol: String, pred: PredHeader, div: [u8; 16]) -> CompactOutput {
    CompactOutput {
        dest: dest.clone(), eph_pk: eph(b"eph-t", &div, &dest.bytes), diversifier: div,
        value_commitment: commit_with_asset(value, blind, &ORTH), pred, asset, symbol,
    }
}

pub fn inventory_blind(spec: &CurveSpec) -> Scalar {
    if spec.quote_raised == 0 {
        blinding_from_seed(&[b"inv-r".as_ref(), &spec.asset, &spec.cap.to_le_bytes()].concat())
    } else {
        blinding_from_seed(&[b"inv-r".as_ref(), &spec.asset, &spec.cap_remaining.to_le_bytes(), &spec.quote_raised.to_le_bytes()].concat())
    }
}

pub fn inventory_cm(spec: &CurveSpec) -> [u8; 32] {
    commit_with_asset(spec.cap_remaining, &inventory_blind(spec), &ORTH).commitment
}

pub fn birth_outputs(
    creator: &SpendKey, symbol: &str, salt: &[u8], cap: u64, virtual_quote: u64,
    graduate_quote: u64, start_height: u64, team_value: u64, team_unlock: u64, height: u64,
) -> Result<(ActionBundle, CurveSpec, [u8; 32]), &'static str> {
    let _ = height; let _ = salt;
    let sym = normalize_symbol(symbol)?.to_string();
    let asset = ticker_id(&sym)?;
    if cap == 0 || virtual_quote == 0 { return Err("cap/virtual"); }
    let spec = CurveSpec::new(asset, cap, virtual_quote, graduate_quote, start_height);
    let inv_r = inventory_blind(&spec);
    let inv = note(creator.pk(), cap, &inv_r, asset, sym.clone(), PredHeader::from_pred(&spec.as_pred()), [1u8; 16]);
    let mut actions = vec![CompactAction { spend: None, output: Some(inv) }];
    if team_value > 0 {
        if team_value >= cap { return Err("team >= cap"); }
        let team_pred = Predicate::After { height: team_unlock };
        let team_r = blinding_from_seed(&[b"team-r".as_ref(), &asset].concat());
        actions.push(CompactAction { spend: None, output: Some(note(
            creator.pk(), team_value, &team_r, asset, sym.clone(), PredHeader::from_pred(&team_pred), [2u8; 16],
        )) });
    }
    let intent = Intent { id: asset, want_asset: asset, pay_asset: ORTH, expire_height: u64::MAX, bound: spec.as_pred().commit() };
    Ok((ActionBundle {
        version: 7, actions, fee_commitment: ValueCommitment::identity(),
        binding: None, emission: None,
        intents: vec![intent], fills: vec![IntentFill { intent_id: asset, action_index: 0 }], exec: None,
    }, spec, asset))
}

pub fn buy_outputs(
    keeper: &SpendKey, buyer: &SpendKey, spec: &CurveSpec, symbol: &str, quote_in: u64, height: u64,
) -> Result<(ActionBundle, CurveSpec, u64), &'static str> {
    if height < spec.start_height { return Err("curve closed"); }
    let (next, tokens) = spec.after_buy(quote_in)?;
    let sym = normalize_symbol(symbol)?.to_string();
    if ticker_id(&sym)? != spec.asset { return Err("symbol/asset mismatch"); }
    let inv_r = inventory_blind(&next);
    let buy_r = blinding_from_seed(&[b"buy-r".as_ref(), &spec.asset, &tokens.to_le_bytes(), &quote_in.to_le_bytes()].concat());
    let inv = note(keeper.pk(), next.cap_remaining, &inv_r, spec.asset, sym.clone(), PredHeader::from_pred(&next.as_pred()), [3u8; 16]);
    let got = note(buyer.pk(), tokens, &buy_r, spec.asset, sym, PredHeader::pk(), [4u8; 16]);
    let intent = Intent {
        id: blinding_from_seed(&[b"buy-id".as_ref(), &spec.asset, &quote_in.to_le_bytes()]).to_bytes(),
        want_asset: spec.asset, pay_asset: ORTH, expire_height: height.saturating_add(64), bound: next.as_pred().commit(),
    };
    let id = intent.id;
    Ok((ActionBundle {
        version: 7,
        actions: vec![
            CompactAction { spend: None, output: Some(inv) },
            CompactAction { spend: None, output: Some(got) },
        ],
        fee_commitment: ValueCommitment::identity(), binding: None, emission: None,
        intents: vec![intent], fills: vec![IntentFill { intent_id: id, action_index: 1 }], exec: None,
    }, next, tokens))
}

/// Spend the live inventory note, emit next inventory + buyer tokens.
/// Token units conserve: rem = rem' + tokens.
pub fn buy_spend(
    keeper: &SpendKey, buyer: &SpendKey, set: &LaunchSet, spec: &CurveSpec,
    symbol: &str, quote_in: u64, height: u64,
) -> Result<(ActionBundle, CurveSpec, u64), &'static str> {
    if height < spec.start_height { return Err("curve closed"); }
    let (next, tokens) = spec.after_buy(quote_in)?;
    let sym = normalize_symbol(symbol)?.to_string();
    if ticker_id(&sym)? != spec.asset { return Err("symbol/asset mismatch"); }
    let in_blind = inventory_blind(spec);
    let note_cm = inventory_cm(spec);
    if !set.contains(note_cm) { return Err("inventory not live"); }
    let pred = spec.as_pred();
    let header = PredHeader::from_pred(&pred);
    let leaves = set.live_leaves_for(header.id, header.commit);
    let idx = leaves.iter().position(|l| *l == note_cm).ok_or("inventory not in curve slice")?;
    let expected = commit_with_asset(spec.cap_remaining, &in_blind, &ORTH);
    if expected.commitment != note_cm { return Err("opening mismatch"); }
    let delta = blinding_from_seed(&[b"rerand".as_ref(), &keeper.sk.to_bytes(), &note_cm].concat());
    let c_prime = rerand(&expected, &delta);
    let image = keeper.key_image(&note_cm);
    let mut ctx = Vec::new();
    ctx.extend_from_slice(&set.window_root());
    ctx.extend_from_slice(&height.to_le_bytes());
    let image_or = ImageOr::prove(&leaves, idx, keeper, &delta, &image, &c_prime, &ctx)?;
    let inv_r = inventory_blind(&next);
    let buy_r = blinding_from_seed(&[b"buy-r".as_ref(), &spec.asset, &tokens.to_le_bytes(), &quote_in.to_le_bytes()].concat());
    let inv = note(keeper.pk(), next.cap_remaining, &inv_r, spec.asset, sym.clone(), PredHeader::from_pred(&next.as_pred()), [3u8; 16]);
    let got = note(buyer.pk(), tokens, &buy_r, spec.asset, sym, PredHeader::pk(), [4u8; 16]);
    let fee_c = ValueCommitment::identity();
    let fee_blind = Scalar::ZERO;
    let r_bind = (in_blind + delta) - inv_r - buy_r - fee_blind;
    let mut transcript = Vec::new();
    transcript.extend_from_slice(&image);
    transcript.extend_from_slice(&inv.value_commitment.commitment);
    transcript.extend_from_slice(&got.value_commitment.commitment);
    transcript.extend_from_slice(&fee_c.commitment);
    transcript.extend_from_slice(&set.window_root());
    transcript.extend_from_slice(&height.to_le_bytes());
    let binding = BindingSig::sign(&r_bind, &transcript)?;
    let proof = NoteProof {
        image: image_or,
        range_in: RangeProof::prove(spec.cap_remaining, &(in_blind + delta)),
        range_outs: vec![RangeProof::prove(next.cap_remaining, &inv_r), RangeProof::prove(tokens, &buy_r)],
        range_fee: RangeProof::prove(0, &fee_blind),
        binding: binding.clone(),
    };
    let mut fill = Vec::new();
    fill.extend_from_slice(&quote_in.to_le_bytes());
    fill.extend_from_slice(&tokens.to_le_bytes());
    let spend = CompactSpend {
        spend_tag: image, rerand: c_prime, proof: Some(proof),
        pred: header, pred_body: Some(pred),
        pred_witness: PredWitness { keys: vec![], fill },
    };
    let bundle = ActionBundle {
        version: 7,
        actions: vec![
            CompactAction { spend: Some(spend), output: Some(inv) },
            CompactAction { spend: None, output: Some(got) },
        ],
        fee_commitment: fee_c, binding: Some(binding), emission: None,
        intents: vec![], fills: vec![], exec: None,
    };
    if !bundle.verify_conservation() { return Err("conservation failed"); }
    Ok((bundle, next, tokens))
}

pub fn sell_outputs(
    keeper: &SpendKey, seller: &SpendKey, spec: &CurveSpec, symbol: &str, tokens_in: u64, height: u64,
) -> Result<(ActionBundle, CurveSpec, u64), &'static str> {
    let _ = height;
    let (next, quote) = spec.after_sell(tokens_in)?;
    let sym = normalize_symbol(symbol)?.to_string();
    if ticker_id(&sym)? != spec.asset { return Err("symbol/asset mismatch"); }
    let inv_r = inventory_blind(&next);
    let q_r = blinding_from_seed(&[b"sell-r".as_ref(), &spec.asset, &quote.to_le_bytes()].concat());
    let inv = note(keeper.pk(), next.cap_remaining, &inv_r, spec.asset, sym, PredHeader::from_pred(&next.as_pred()), [5u8; 16]);
    let pay = note(seller.pk(), quote, &q_r, ORTH, String::new(), PredHeader::pk(), [6u8; 16]);
    Ok((ActionBundle {
        version: 7,
        actions: vec![
            CompactAction { spend: None, output: Some(inv) },
            CompactAction { spend: None, output: Some(pay) },
        ],
        fee_commitment: ValueCommitment::identity(), binding: None, emission: None,
        intents: vec![], fills: vec![], exec: None,
    }, next, quote))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notes::keys::SpendKey;
    use crate::notes::launch::LiveNote;
    #[test]
    fn buy_spend_kills_old_inventory() {
        let k = SpendKey::from_wallet_seed(b"k");
        let b = SpendKey::from_wallet_seed(b"b");
        let (birth, spec, _) = birth_outputs(&k, "MEME", b"s", 1_000_000, 10_000, 80_000, 1, 0, 0, 1).unwrap();
        let mut set = LaunchSet::standard();
        for n in birth.output_records() { set.append_note(n); }
        let cm = inventory_cm(&spec);
        assert!(set.contains(cm));
        let (bundle, next, tokens) = buy_spend(&k, &b, &set, &spec, "MEME", 1_000, 2).unwrap();
        assert!(tokens > 0);
        assert_eq!(bundle.real_spends().len(), 1);
        assert_eq!(bundle.real_outputs().len(), 2);
        assert_ne!(inventory_cm(&next), cm);
        let _ = LiveNote::plain(cm);
    }
}
