# Test Taxonomy

Atlasctl test modules must align with this taxonomy:

- `unit`: fast pure behavior checks.
- `contract`: schema, CLI surface, and output contract checks.
- `golden`: snapshot/golden output checks.
- `integration`: command-level integration checks.
- `repo-sim`: repository simulation checks using sandbox fixtures.

## Placement Rules

- Every test file must live under `packages/atlasctl/tests/<domain>/test_*.py`.
- A test file may opt into taxonomy explicitly with a header comment:
  - `# test-taxonomy: unit|contract|golden|integration|repo-sim`
- Without an explicit header, taxonomy is inferred:
  - `tests/checksuite/contracts/*` -> `contract`
  - `tests/goldens/*` and files named `*golden*` -> `golden`
  - `tests/opssuite/integration/*` -> `integration`
  - `tests/policysuite/repo/*` and `tests/policysuite/inventory/*` -> `repo-sim`
  - all others -> `unit`

## Guardrails

- Network is forbidden in tests unless explicitly marked (`@pytest.mark.network` or `# network-test: allowed`).
- Writes must stay inside pytest temp dirs or `artifacts/isolate/<run_id>/...`.
- Golden updates must flow only through `atlasctl gen goldens`.
- Determinism checks forbid unstable time/random/environment patterns without local justification markers.
