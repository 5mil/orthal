# Orthal developer launch (testnet)

Source: `github.com/5mil/orthal` `main` — crate `hybrid-chain`.
Lab mirror: `github.com/5mil/solana` `dev`.

## Pin checks

```bash
git clone --branch main https://github.com/5mil/orthal.git
cd orthal/hybrid-chain
cargo fmt --all -- --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
cargo test mine_birth_buy_sell -- --nocapture
```

## PoW difficulty

`d` is leading-zero bits on SHA256d(header). Genesis `d = 12`. Window is 2016 blocks × 120s. Actual span is clamped to `[T/4, 4T]`. Retarget steps `d` by the number of doublings between actual and target (2× fast → +1 bit, 4× fast → +2). Floor is 12, cap is 240.

## Tickers

```text
let (birth, spec, _) = birth_outputs(&keeper, "MEME", b"", cap, virt, grad, 0, 0, 0, h)?;
let birth = with_genesis_vault(birth, &keeper, &spec);
chain.apply_transfer(&birth)?;

let (buy, spec, tokens) = buy_fund(&keeper, &buyer, &chain.launch, &spec, "MEME", quote, h, orth_cm, orth_r)?;
chain.apply_transfer(&buy)?;

let (sell, spec, paid) = sell_spend(&keeper, &seller, &chain.launch, &spec, "MEME", tokens, h, tok_cm, tok_r)?;
chain.apply_transfer(&sell)?;

if spec.graduated() {
    let g = graduate_spend(&keeper, &chain.launch, &spec, "MEME", h)?;
    chain.apply_transfer(&g)?;
}
```

Sells pay from the vault. `graduate_spend` turns remaining inventory + vault into `PoolLp` notes. Curve buys refuse after graduate.

## Honesty

48-bit range proofs are a prototype. Relay is in-process. Not mainnet.
