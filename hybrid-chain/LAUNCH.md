# Orthal developer launch (testnet)

Source of truth: `github.com/5mil/orthal` `main`, crate `hybrid-chain`.
Lab mirror: `github.com/5mil/solana` `dev`.

This is a private testnet. Do not announce payroll or mainnet.

## 1. Pin checks

```bash
git clone --branch main https://github.com/5mil/orthal.git
cd orthal/hybrid-chain
cargo fmt --all -- --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
cargo test mine_birth_buy_sell -- --nocapture
```

Go only if all four are green.

## 2. Roles and Discord

| Role | Does | Room |
| --- | --- | --- |
| Lead | Cuts the pin, posts go/no-go | `#launch-control` |
| Node ops | Build, mine, persist, replay | `#nodes` |
| Pool ops | Accept a share; never hold the seed | `#pool` |
| Ticker ops | Birth, vault, buy_fund, sell_spend, graduate | `#tickers` |
| Review | fmt / test / clippy / flow | `#ci` |
| Announce | Webhook + one human summary | `#announce` |

Webhook on `#announce` only. Never post seeds, blinds, or spend tags.

```bash
export DISCORD_WEBHOOK='https://discord.com/api/webhooks/…'
curl -sS -H 'Content-Type: application/json' \
  -d '{"content":"Orthal pin orthal@main — cargo test green, flow test green"}' \
  "$DISCORD_WEBHOOK"
```

## 3. Node, mine, persist

Isolated data dir. One wallet seed per operator, generated locally.

```text
install -d ~/orthal-data
export ORTHAL_DATA=$HOME/orthal-data/chain.bin
cargo run --release -- --data "$ORTHAL_DATA"
```

Payout dest comes from a wallet seed, not a pool ticket:

```text
let pay = SealedPayout::from_wallet_seed(b"ops-wallet", chain.height());
let block = chain.mine_pow_with_payout("pool-ticket", &pay, vec![]);
```

Save, reload, same height and tip. A tampered nonce or parent must refuse load.
A missing file is a hard error, not an empty chain.

## 4. PoW difficulty

`d` = leading-zero bits on `SHA256d(header)`. Genesis `d = 12`.
Window = 2016 blocks × 120 seconds. Actual span clamped to `[T/4, 4T]`.
Retarget steps `d` by doublings between actual and target:

| Window vs target | Change in `d` |
| --- | --- |
| 2× fast | +1 bit |
| 4× fast (clamp) | +2 bits |
| 2× slow | −1 bit |
| 4× slow (clamp) | −2 bits |

Floor 12, cap 240. PoS does not move this `d`.

## 5. Tickers

- ORTH is native and not issuable.
- Symbol `A-Z0-9`, 1..=12. Asset id = `sha256d("orthal-ticker" ‖ SYMBOL)`.
- Same name cannot be a second asset. A later fill of that id is a refill.
- Inventory note = remaining tokens under `Curve`.
- Vault note = raised ORTH. Sells pay from here.

```text
let (birth, spec, _) = birth_outputs(&keeper, "MEME", b"", cap, virt, grad, 0, 0, 0, h)?;
let birth = with_genesis_vault(birth, &keeper, &spec);
chain.apply_transfer(&birth)?;

let (buy, spec, tokens) = buy_fund(
    &keeper, &buyer, &chain.launch, &spec, "MEME", quote, h,
    buyer_orth_cm, buyer_orth_blind)?;
chain.apply_transfer(&buy)?;

let (sell, spec, paid) = sell_spend(
    &keeper, &seller, &chain.launch, &spec, "MEME", tokens, h,
    seller_token_cm, seller_token_blind)?;
chain.apply_transfer(&sell)?;

if spec.graduated() {
    let g = graduate_spend(&keeper, &chain.launch, &spec, "MEME", h)?;
    chain.apply_transfer(&g)?;
}
```

`buy_fund` spends inventory + vault + buyer ORTH.
`sell_spend` spends inventory + vault + seller tokens.
`graduate_spend` turns remaining inventory + vault into two `PoolLp` notes.
Curve buys refuse after graduate.

Split a mined reward before `buy_fund` if the quote is smaller than the subsidy.
See `notes/flow.rs` (`mine_birth_buy_sell`).

## 6. Honesty

Claim: hybrid node, unique symbols, vault-backed curve, listed predicates.
Do not claim: production range proofs, public relay, mainnet, payroll.
