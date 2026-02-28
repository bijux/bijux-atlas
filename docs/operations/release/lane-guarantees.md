# Release Lane Guarantees

- Owner: `bijux-atlas-operations`
- Status: `stable`
- Audience: `operators`

## Lane map

- `local`: ad hoc developer runs with no merge-blocking selection.
- `pr`: all required contracts plus static coverage.
- `merge`: required contracts plus effect coverage needed for merge readiness.
- `release`: full matrix coverage across required, effect, and slow checks.

## Commands

- `make contracts-pr`
- `make contracts-merge`
- `make contracts-release`

## Effect truth checks

- `merge`: requires runtime image build, runtime smoke help, helm defaults render, minimal chart render, kubeconform validation, and recorded helm or kubeconform tool versions.
- `release`: adds runtime SBOM generation, vulnerability threshold enforcement, and kind-backed install smoke prerequisites.
- Effect subprocess calls are logged to `artifacts/contracts/ops/effects.log` and `artifacts/contracts/docker/effect/effects.log`.

## Slow lane policy

- Kind-backed and other slow effect checks stay out of PR gating.
- The slow lane runs daily through `.github/workflows/ci-nightly.yml` and `.github/workflows/ops-integration-kind.yml`.
- Nightly artifact retention is governed by `configs/ops/artifact-retention.json`.

## Required contracts

- Source-of-truth: `ops/policy/required-contracts.json`
- Committed artifact: `artifacts/contracts/required.json`
- Approval metadata: `ops/policy/required-contracts-change.json`
- Merge and release wall: `docs/operations/release/quality-wall.md`
