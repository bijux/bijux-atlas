# Boundary Closure Checks

- Owner: `bijux-atlas-governance`
- Type: `runbook`
- Audience: `reviewers`
- Stability: `stable`
- Last verified against: `main@c59da0bf`
- Reason to exist: keep the repository closed at the boundaries where drift previously escaped review.

## Boundary closure: Helm env subset

- Check ID: `REPO-003`
- Guarantee: rendered Helm env keys must stay inside `configs/contracts/env.schema.json`.
- Reproduce:
  - `bijux dev atlas contracts repo --mode effect --allow-subprocess --filter-contract REPO-003`
  - `bijux dev atlas ops helm-env --chart ops/k8s/charts/bijux-atlas --values ops/k8s/charts/bijux-atlas/values.yaml --allow-subprocess --format json`

## Boundary closure: ops profile matrix

- Check IDs: `REPO-004`, `OPS-PROFILES-001`, `OPS-PROFILES-002`, `OPS-PROFILES-003`, `OPS-PROFILES-004`
- Guarantee: every profile in `ops/k8s/install-matrix.json` remains installable by construction and every rollout-safety profile exists.
- Reproduce:
  - `bijux dev atlas contracts repo --mode effect --allow-subprocess --filter-contract REPO-004`
  - `bijux dev atlas ops profiles validate --allow-subprocess --profile-set rollout-safety --format json`

## Boundary closure: docs site output

- Check IDs: `REPO-005`, `DOCS-SITE-001`, `DOCS-SITE-002`, `DOCS-SITE-003`
- Guarantee: `mkdocs build --strict` publishes to the configured `site_dir`, and that output contains `index.html`, assets, and a non-trivial file count.
- Reproduce:
  - `bijux dev atlas contracts repo --mode effect --allow-subprocess --filter-contract REPO-005`
  - `bijux dev atlas docs site-dir --format json`

## Boundary closure: runtime env allowlist enforcement

- Runtime guard: `atlas-server --validate-config`
- Guarantee: unknown `ATLAS_*` and `BIJUX_*` env keys fail startup unless `ATLAS_DEV_ALLOW_UNKNOWN_ENV=1` is set explicitly for local development.
- Reproduce:
  - `cargo test -p bijux-atlas-server --test runtime_env_contract_startup`
  - `cargo test -p bijux-atlas-server --release --test runtime_env_contract_startup`

## Why this exists

These checks close the exact drift classes found during the audits:

- Helm emitted environment variables that were outside the runtime contract.
- Rollout profiles that rendered locally but were not installable by construction.
- Docs workflows that copied a preview directory that had drifted away from the MkDocs source of truth.

## How to reproduce locally

- `bijux dev atlas contracts repo --mode effect --allow-subprocess --filter-contract REPO-003`
- `bijux dev atlas contracts repo --mode effect --allow-subprocess --filter-contract REPO-004`
- `bijux dev atlas contracts repo --mode effect --allow-subprocess --filter-contract REPO-005`
- `bijux dev atlas contracts repo --mode effect --allow-subprocess --filter-contract REPO-006`

Reports are written under `artifacts/contracts/repo/boundary-closure/`.

## Lane policy

The closure checks are marked as blocking for `pr`, `merge`, and `release` in `ops/policy/required-contracts.json`.
CI must call the control-plane contract runner; it must not reimplement these checks with ad hoc workflow scripts.
