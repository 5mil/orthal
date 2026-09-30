# Orthal (`ORTH`)

Hybrid SHA256d proof-of-work and coin-age proof-of-stake. Compact action
bundles. Native unit **ORTH**. Issued tickers are notes on the same ledger —
not a second virtual machine and not a deployed token program.

Product repo: [github.com/5mil/orthal](https://github.com/5mil/orthal) (`main`).
Lab mirror: [5mil/solana:dev](https://github.com/5mil/solana/tree/dev).

```text
mine ORTH  →  birth + vault  →  buy_fund  →  sell_spend  →  graduate_spend
```

## Consensus

PoW uses a 256-bit **target** on header v5. A header is valid when
`SHA256d(header) ≤ target`. The next PoW target is ASERT against genesis:

```text
next = genesis_target × 2^((t - t0 - n·τ) / half_life)
```

`τ = 120s`, half-life = 2 days. Samples are **PoW-only**. Timestamps must beat
the median of the last 11 PoW times. Accumulated `chain_work` is the fork-choice
key. PoS mint does not change the PoW target and adds no work.

Set `genesis_timestamp` to the real launch instant before a public pin.

## Tickers and programs

See the tables in the previous README revision: unique symbols, vault-backed
curve, listed predicates. Helpers live in `notes/trade.rs`, `notes/graduate.rs`,
`notes/market.rs`.

## Build

```bash
git clone --branch main https://github.com/5mil/orthal.git
cd orthal/hybrid-chain
cargo test --all-targets
cargo run --release -- --data /tmp/orthal/chain.bin
```

Operator pin: [LAUNCH.md](LAUNCH.md).
