# Quality wall

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last reviewed: `2026-03-01`
- Reason to exist: tie required contracts, release lanes, and repository control surfaces into one release gate.

## Required contract surfaces

- `ops/policy/required-contracts.json`
- `ops/_generated.example/contracts-required.json`
- `make/contracts.mk`
- `docker/images/runtime/Dockerfile`
- `docs/_internal/generated/make-targets.md`
- `configs/contracts/env.schema.json`
- `docs/reference/contracts/schemas/CONFIG_KEYS.json`

## Release lanes

- `local`: contributor loop and deterministic preflight.
- `pr`: pull request gate with required suites.
- `merge`: protected-branch merge gate with full required checks.
- `release`: release candidate gate with ops and docker truth checks.

