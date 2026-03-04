# Readiness Semantics

Readiness must answer whether the node can safely serve production traffic.

Readiness components:

- runtime admission checks
- dataset/cache availability checks
- overload guard status
- configured readiness policy gates

Expected behavior:

- `200` means traffic can proceed.
- `503` means traffic should be held or drained.
