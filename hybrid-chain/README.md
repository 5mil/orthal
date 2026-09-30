# Orthal (`ORTH`)

Hybrid SHA256d proof-of-work and coin-age proof-of-stake. Compact action
bundles. Native unit **ORTH**. Issued tickers are notes on the same ledger —
not a second virtual machine and not a deployed token program.

Product repo: [github.com/5mil/orthal](https://github.com/5mil/orthal) (`main`).
Lab mirror: [5mil/solana:dev](https://github.com/5mil/solana/tree/dev).

```text
mine ORTH  →  birth + vault  →  buy_fund  →  sell_spend  →  graduate_spend
```

## What you can do with it

| You want | You use |
| --- | --- |
| A mineable coin named ORTH | `mine_pow_with_payout(ticket, &wallet, extra)` |
| A named ticker that cannot be cloned | `birth_outputs` + `AssetBook`. Symbol unique forever |
| An empty quote vault at birth | `with_genesis_vault` |
| A buy that deposits ORTH | `buy_fund` (inventory + vault + buyer ORTH) |
| A sell that pays from deposits | `sell_spend` (inventory + vault + seller tokens) |
| A fair / locked / deep launch | `LaunchPreset::{fair, locked_team, deep}` |
| Off-node matching | `IntentBoard` (one leg must be ORTH) |
| Graduate the curve | `graduate_spend` → two `PoolLp` notes |
| Time locks, multisig, swaps | Listed predicates on the note |
| A node that refuses a bad file | `--data` then `--replay` |

## Native unit

Orthal / **ORTH** / 21,000,000 cap / 50 ORTH PoW subsidy / 120s PoW, 60s PoS.
`ORTH` cannot be issued as a ticker (`asset = [0u8; 32]`).

PoW difficulty `d` is leading-zero bits on `SHA256d(header)`. Genesis `d = 12`.
Every 2016 blocks the node compares the window to `2016 × 120s`, clamps the
span to `[T/4, 4T]`, and steps `d` by the number of doublings (2× fast → +1 bit).
Floor 12, cap 240.

## Issued tickers

Asset id is `sha256d("orthal-ticker" ‖ SYMBOL)`. `A–Z0–9`, length 1–12.
The same name always hashes to the same id. A later fill of that id is a
refill, not a second asset.

Curve (virtual product):

```text
tokens_out = rem × q_in / (virtual + raised + q_in)
```

Raised ORTH lives in a **vault note**. Sells cannot mint quote. After
`quote_raised >= graduate_quote`, `buy` refuses and `graduate_spend` converts
inventory + vault into `PoolLp` notes.

Helpers: `notes/market.rs` (`preview_buy`, presets, `IntentBoard`).
On-chain path: `notes/trade.rs`, `notes/graduate.rs`.
Flow test: `cargo test mine_birth_buy_sell`.

## Programs

Listed predicates only: `Pk`, `PkN`, `After`, `And`/`Or`, `Rate`, `Swap`,
`PoolLp`, `Curve`, `Ticker`. Unknown ids fail closed. No bytecode VM.
See [PROGRAM.md](PROGRAM.md).

## Build

```bash
git clone --branch main https://github.com/5mil/orthal.git
cd orthal/hybrid-chain
cargo fmt --all -- --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
cargo run --release -- --data /tmp/orthal/chain.bin
cargo run --release -- --data /tmp/orthal/chain.bin --replay
```

Operator pin and Discord: [LAUNCH.md](LAUNCH.md).
Living-set notes: [NOTES.md](NOTES.md).
