# Living-set notes

- Native ticker is ORTH. Issued tickers are notes with a public asset id.
- Asset id is H(symbol). A name cannot be reissued. ORTH is reserved.
- Curve: tokens_out = rem * q_in / (virtual + raised + q_in). State on the inventory note.
- Birth is issuance, not coinbase. Team slice uses After.
- Listed preds include curve and ticker. Unknown id fails.
- Conservation is BindingSig: residual r ≠ 0.
