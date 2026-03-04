# Data Access Model

- Owner: `bijux-atlas-data`
- Type: `concept`
- Audience: `reviewers`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Last changed: `2026-03-03`
- Reason to exist: describe who may access dataset sources and derived artifacts.

Related ops contracts: `OPS-ROOT-023`, `REL-DATA-001`.

## Access Rules

- Dataset source material is controlled input and must be explicitly declared in `configs/datasets/manifest.yaml`.
- Operators may read governed fixtures and generated ingest evidence.
- Release reviewers may inspect retained dataset evidence through the review packet and evidence bundle.
- No runtime path should require ad hoc undisclosed dataset access outside the governed manifest.

## Verify

- Every dataset used by the offline profile is declared in the manifest and pinned policy.
- Review surfaces point to committed or generated evidence, not undocumented storage.
