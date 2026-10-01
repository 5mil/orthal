# Orthal

**Repo:** [github.com/5mil/orthal](https://github.com/5mil/orthal) (`main`)
**Crate:** [`hybrid-chain`](hybrid-chain/)
**Unit:** ORTH
**Lab mirror:** [5mil/solana:dev](https://github.com/5mil/solana/tree/dev)

Hybrid SHA256d PoW / coin-age PoS. Transfers, miner payouts, issued tickers,
and listed predicates share one compact `ActionBundle`. There is no token-factory VM.

```text
mine ORTH → birth a unique ticker → buy_fund / sell_spend → graduate to PoolLp
```

Developer start:

1. [hybrid-chain/README.md](hybrid-chain/README.md) — product surface
2. [hybrid-chain/LAUNCH.md](hybrid-chain/LAUNCH.md) — pin, mine, persist, Discord
3. [hybrid-chain/PROGRAM.md](hybrid-chain/PROGRAM.md) — predicates and intents
4. [hybrid-chain/NOTES.md](hybrid-chain/NOTES.md) — living set and conservation

Nix: [NIX.md](NIX.md). `nix develop`, `nix build`, `nix flake check`. Overlay and hardened oneshot module included.

```bash
git clone --branch main https://github.com/5mil/orthal.git
cd orthal/hybrid-chain
cargo test --all-targets
cargo test mine_birth_buy_sell -- --nocapture
```

Testnet only. Range proofs are a 48-bit prototype. Relay is in-process.
