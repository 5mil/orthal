# Launch membership vs full-chain proofs

Full-chain membership proves “this output is one of every output ever.”
Prover work and node RAM grow with the chain.

## What launch does

1. **Hide the window.** Path + leaf. No epoch index on the wire.
2. **Keep epochs populated.** `seal()` pads live to the profile bucket.
3. **Fold.** Header commitment = `H(window_root || forest_acc)`.

`HiddenProof` is a fixed-size path. Notes that leave the window must refresh.
