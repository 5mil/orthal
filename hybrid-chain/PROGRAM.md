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
`ImageOr` runs over `live_leaves_for(id, commit)`.

## Intents

`ActionBundle.intents` + `fills`. The node does not match. It checks expiry
and that each intent has a fill index into this bundle.

## Exec

`exec: Option<ExecProof>`. Only `program_id = 0` with an empty proof is
listed. Any other program id is rejected.
