# Living-set notes

- Native unit is ORTH (`asset = [0;32]`). Issued tickers are notes with a public asset id.
- Asset id is `sha256d("orthal-ticker" ‖ SYMBOL)`. A name cannot be born as a second asset.
  A later fill of the same id is a refill.
- Curve: `tokens_out = rem * q_in / (virtual + raised + q_in)`.
- Inventory value is `cap_remaining`. Vault value is `quote_raised`.
- `birth_outputs` creates the inventory. `with_genesis_vault` adds a zero ORTH vault.
- `buy_fund` deposits buyer ORTH into the vault and emits buyer tokens.
- `sell_spend` pays seller ORTH from the vault. Dry vault fails.
- `graduate_spend` converts inventory + vault into `PoolLp` notes.
- Constructor-only helpers `buy_outputs` / `sell_outputs` do not retire the previous note.
- Listed preds include Curve, Ticker, PoolLp. Unknown id fails closed.
- Conservation is `BindingSig` with residual `r ≠ 0`. Multi-input bundles bind the sum.
