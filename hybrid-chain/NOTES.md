# Living-set notes

- Native ticker is ORTH. Issued tickers are notes with a public asset id.
- Asset id is H(symbol). A name cannot be born twice. A buy/sell refill of the same id is not a second birth.
- Curve: tokens_out = rem * q_in / (virtual + raised + q_in).
- `birth_outputs` creates the inventory. `buy_outputs` / `sell_outputs` emit the next inventory + the counterparty note.
- Listed preds include curve and ticker. Unknown id fails.
- Conservation is BindingSig: residual r ≠ 0.
