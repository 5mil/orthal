# Programmed notes

Value still moves as `ActionBundle`. Programs are predicates on notes, not a
second transaction type and not an account VM.

## Birth

`CompactOutput.pred` is `{id, commit}`. Default (zeros) is spend-key only.
Listed ids: `pk`, `pk-n`, `after`, `and`, `or`, `rate`, `swap`, `pool-lp`,
`curve`, `ticker`. Unknown ids fail closed.

The living set stores `LiveNote { cm, pred, pred_commit }`.

## Spend

`CompactSpend` carries the same header, the predicate body, and a witness.
`ImageOr` runs over `live_leaves_for(id, commit)` — the matching pred slice,
not a listed ring.

One spend: `NoteProof.verify` checks ImageOr, ranges, and a single-input binding.
Many spends (buy_fund, sell_spend, graduate): each leg uses `verify_open`;
the bundle `BindingSig` is over the **sum** of rerands.

## Curve and vault

`Predicate::Curve` holds cap / virtual / graduate / raised / remaining.
`buy_fund` and `sell_spend` require the inventory and the ORTH vault to be live.
`Predicate::PoolLp` is what `graduate_spend` writes after the raise clears
`graduate_quote`. `Curve` spends after that point fail (`"graduated"` / `"not graduated"`).

## Intents

`ActionBundle.intents` + `fills`. The node does not match. It checks expiry
and that each intent has a fill index into this bundle. `IntentBoard` is the
off-node helper; one leg must be ORTH.

## Exec

`exec: Option<ExecProof>`. Only `program_id = 0` with an empty proof is
listed. Any other program id is rejected.
