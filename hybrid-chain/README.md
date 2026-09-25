# Orthal (`ORTH`)

Hybrid SHA256d proof-of-work and coin-age proof-of-stake. Compact action
bundles. Native unit **ORTH**. Issued tickers are notes on the same ledger —
not a second virtual machine and not a deployed token program.

Product repo: [github.com/5mil/orthal](https://github.com/5mil/orthal) (`main`).
Integration mirror: [5mil/solana:dev](https://github.com/5mil/solana/tree/dev).

```text
mine ORTH  →  birth a ticker  →  buy / sell the curve  →  graduate to an LP note
```

## What you can do with it

| You want | You use |
| --- | --- |
| A mineable coin named ORTH | `mine_pow_block(wallet_seed)` |
| A named ticker that cannot be cloned | `birth_outputs` + `AssetBook`. Symbol is unique forever |
| A fair launch | `LaunchPreset::fair()` |
| A founder bag that cannot move yet | `LaunchPreset::locked_team()` |
| A flatter first hour | `LaunchPreset::deep()` |
| Off-node matching | `IntentBoard` (one leg must be ORTH) |
| Time locks, multisig, swaps, LP | Listed predicates on the note at birth |
| A node that refuses a bad file | `--replay` |

## Native unit

Orthal / **ORTH** / 21,000,000 cap / 50 ORTH PoW subsidy / 120s PoW, 60s PoS.
`ORTH` cannot be issued as a ticker.

## Issued tickers

Asset id is `H("orthal-ticker" ‖ SYMBOL)`. Same name cannot be born twice.
Curve: `tokens_out = rem × q_in / (virtual + raised + q_in)`.
Helpers: `notes/market.rs` (`preview_buy`, presets, board).

## Programs

Listed predicates only: `Pk`, `PkN`, `After`, `And`/`Or`, `Rate`, `Swap`,
`PoolLp`, `Curve`, `Ticker`. Unknown ids fail closed. No bytecode VM.

## Build

```bash
git clone https://github.com/5mil/orthal.git
cd orthal/hybrid-chain
cargo test --all-targets
cargo run -- --data /tmp/orthal/chain.bin
cargo run -- --data /tmp/orthal/chain.bin --replay
```

Further reading: [NOTES.md](NOTES.md).
