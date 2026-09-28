//! End-to-end ticker flow on a live chain.

#[cfg(test)]
mod tests {
    use crate::chain::blockchain::Blockchain;
    use crate::notes::commitment::blinding_from_seed;
    use crate::notes::keys::SpendKey;
    use crate::notes::payout::SealedPayout;
    use crate::notes::proof::commit_with_asset;
    use crate::notes::spend::transfer_window_bundle;
    use crate::notes::ticker::birth_outputs;
    use crate::notes::trade::{buy_fund, sell_spend, with_genesis_vault};
    use crate::notes::ORTH;
    use crate::params::CHAIN_PARAMS;

    #[test]
    fn mine_birth_buy_sell() {
        let keeper = SpendKey::from_wallet_seed(b"keeper");
        let mut chain = Blockchain::new();
        let h_mine = chain.height();
        let pay = SealedPayout::from_wallet_seed(b"buyer-wallet", h_mine);
        let _ = chain.mine_pow_with_payout("local", &pay, Vec::new());
        let reward = CHAIN_PARAMS.pow_block_reward;
        let orth_r = blinding_from_seed(&[b"emit-r".as_ref(), &pay.spend.sk.to_bytes(), &h_mine.to_le_bytes()].concat());
        let orth_cm = commit_with_asset(reward, &orth_r, &ORTH).commitment;
        assert!(chain.launch.contains(orth_cm), "mined payout must be live");

        let quote = 10_000u64;
        let q_r = blinding_from_seed(b"quote-r");
        let f_r = blinding_from_seed(b"fee-r");
        let split = transfer_window_bundle(
            &pay.spend, &pay.scan, &chain.launch, orth_cm, reward, &orth_r,
            &pay.spend, quote, &q_r, reward - quote, &f_r, [4u8; 16], chain.height(),
        ).expect("split reward");
        chain.apply_transfer(&split).expect("apply split");
        let quote_cm = commit_with_asset(quote, &q_r, &ORTH).commitment;

        let (birth, spec, _) = birth_outputs(&keeper, "MEME", b"", 1_000_000, 10_000, 80_000, 0, 0, 0, chain.height()).unwrap();
        let birth = with_genesis_vault(birth, &keeper, &spec);
        chain.apply_transfer(&birth).expect("birth+vault");

        let (buy, spec, tokens) = buy_fund(
            &keeper, &pay.spend, &chain.launch, &spec, "MEME", quote, chain.height(),
            quote_cm, q_r,
        ).expect("buy_fund");
        assert!(tokens > 0);
        chain.apply_transfer(&buy).expect("apply buy");

        let tok_r = blinding_from_seed(&[b"buy-r".as_ref(), &spec.asset, &tokens.to_le_bytes(), &quote.to_le_bytes()].concat());
        let tok_cm = commit_with_asset(tokens, &tok_r, &ORTH).commitment;

        let (sell, spec2, paid) = sell_spend(
            &keeper, &pay.spend, &chain.launch, &spec, "MEME", tokens, chain.height(),
            tok_cm, tok_r,
        ).expect("sell_spend");
        assert!(paid > 0 && paid <= quote);
        chain.apply_transfer(&sell).expect("apply sell");
        assert!(spec2.cap_remaining > spec.cap_remaining);
    }
}
