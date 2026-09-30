# Orthal developer launch (testnet → mainnet prep)

Source of truth: `github.com/5mil/orthal` `main`, crate `hybrid-chain`.
Lab mirror: `github.com/5mil/solana` `dev`.

Header version **5**. Old `chain.bin` files do not load. Start a fresh data dir.
Set `CHAIN_PARAMS.genesis_timestamp` to launch time before a public pin — ASERT
is anchored there.

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

## 2. PoW target (ASERT)

Validity is `SHA256d(header) ≤ target` with `target` a 256-bit big-endian integer
on the header. Genesis target is 12 leading zero bits.

Each new **PoW** header:

```text
time_error = (ts - genesis_ts) - pow_count × 120s
next_target = genesis_target × 2^(time_error / 172800s)
```

Half-life is two days. Fast blocks shrink the target; a year of downtime grows it
up to `min_pow_bits = 8`. Cap `max_pow_bits = 240`.

- Only PoW headers feed ASERT. PoS mint copies the last PoW target and adds **zero** work.
- Timestamp must be after median of last 11 PoW times, and not more than 2h past the miner's clock.
- `chain_work` accumulates `2^{bits(target)}` on PoW blocks. Heavier work wins; height does not.
- Replay recomputes ASERT and refuses a header whose target does not match.

PoS stays coin-age (`StakeProof`). It does not have a hash target yet.

## 3. Node, mine, persist

```text
install -d ~/orthal-data
export ORTHAL_DATA=$HOME/orthal-data/chain.bin
cargo run --release -- --data "$ORTHAL_DATA"
cargo run --release -- --data "$ORTHAL_DATA" --replay
```

Payout dest comes from a wallet seed, not a pool ticket.

## 4. Tickers

`birth_outputs` + `with_genesis_vault` → `buy_fund` → `sell_spend` → `graduate_spend`.
See [README.md](README.md) and [NOTES.md](NOTES.md).

## 5. Honesty

ASERT + MTP + work are the mainnet *shape*. Still prototype: 48-bit range proofs,
in-process relay, no peer protocol. Not payroll.
