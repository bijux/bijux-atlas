# Quality Wall

- Owner: `bijux-atlas-operations`
- Status: `stable`
- Audience: `operators`

## What

Merge and release readiness wall for the repository control plane.

## Lane map

- `local`: developer validation without merge-blocking selection.
- `pr`: required contracts and static coverage.
- `merge`: required contracts plus effect coverage.
- `release`: required contracts plus effect and slow coverage.

## Required contracts

- Source of truth: `ops/policy/required-contracts.json`
- Committed manifest: `artifacts/contracts/required.json`
- Approval record for required-set changes: `ops/policy/required-contracts-change.json`
- Public gate entrypoint: `make/contracts.mk`

## Repo integration wall

- Root container entrypoint stays canonical through `Dockerfile` -> `docker/images/runtime/Dockerfile`.
- Public make gate targets must stay documented in `docs/_generated/make-targets.md`.
- Runtime config contracts stay joined through `configs/contracts/env.schema.json` and `docs/contracts/CONFIG_KEYS.json`.
- Release and merge decisions use the same lane vocabulary documented in `docs/operations/release/lane-guarantees.md`.

## Merge checklist

- `make contracts-merge`
- required contracts have zero failures
- runtime image build and smoke coverage pass
- Helm render and manifest validation pass

## Release checklist

- `make contracts-release`
- required contracts have zero failures
- effect and slow coverage pass
- SBOM and vulnerability threshold checks pass

## Failure policy

- Required contract failures are stop-ship.
- Required set changes need explicit approval metadata before merge.
- Contract gate definitions must remain centralized in `make/contracts.mk`.

## See also

- [Release Lane Guarantees](lane-guarantees.md)
- [Operations Index](../INDEX.md)
