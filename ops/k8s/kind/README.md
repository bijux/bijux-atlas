# Kind Simulation Cluster

- Owner: `bijux-atlas-operations`
- Review cadence: `quarterly`
- Type: `guide`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@55054ec60b03589b6f7f52da4d58a6def04d1e2e`
- Last changed: `2026-03-03`
- Reason to exist: define the deterministic kind cluster used by install and smoke simulation.

## Purpose

This cluster is the canonical local simulation target for `ops kind`, `ops helm`, and `ops smoke`.

## Constraints

- The cluster name is fixed to `bijux-atlas-sim`.
- The current implementation assumes no external network requirement beyond tool access.
- The control-plane node is labeled `bijux.atlas/simulation=true` so operators can identify the
  simulation surface clearly.

## Verification

- `kind get clusters` includes `bijux-atlas-sim`.
- `kubectl --context kind-bijux-atlas-sim get nodes` returns at least one ready node.
