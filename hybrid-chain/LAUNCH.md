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

## Node

Isolated data dir. Wallet seed is local. Pool tickets are labels, not keys.

```text
let pay = SealedPayout::from_wallet_seed(b"ops-wallet", chain.height());
let block = chain.mine_pow_with_payout("pool-ticket", &pay, vec![]);
```

Save, reload, same height and tip. Tampered nonce must refuse load.

## Tickers

- ORTH is native (`asset = [0;32]`), not issuable.
- Symbol is `A-Z0-9`, 1..=12. Asset id = `sha256d("orthal-ticker" || SYMBOL)`.
- Same name cannot be born as a second asset. A later fill of that id is a refill.
- Curve: `tokens_out = rem * q_in / (virtual + raised + q_in)`.
- Vault holds raised ORTH. Sells pay from the vault.

```text
let (birth, spec, _) = birth_outputs(&keeper, "MEME", b"", cap, virt, grad, 0, 0, 0, h)?;
let birth = with_genesis_vault(birth, &keeper, &spec);
chain.apply_transfer(&birth)?;

let (buy, spec, tokens) = buy_fund(&keeper, &buyer, &chain.launch, &spec, "MEME", quote, h, orth_cm, orth_r)?;
chain.apply_transfer(&buy)?;

let (sell, spec, paid) = sell_spend(&keeper, &seller, &chain.launch, &spec, "MEME", tokens, h, tok_cm, tok_r)?;
chain.apply_transfer(&sell)?;
```

## Discord

Rooms: `#launch-control` `#nodes` `#pool` `#tickers` `#ci` `#announce`.
Webhook on `#announce` only. Never post seeds, blinds, or key images.

## Honesty

48-bit range proofs are a prototype. Relay is in-process. This is not mainnet and not payroll.
