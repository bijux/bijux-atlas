# Boundary Closure Checks

- Owner: `bijux-atlas-governance`
- Type: `runbook`
- Audience: `reviewers`
- Stability: `stable`
- Last verified against: `main@c59da0bf`
- Reason to exist: keep the repository closed at the boundaries where drift previously escaped review.

## What the closure checks cover

- Helm env surface closure: rendered Helm env keys must stay inside `configs/contracts/env.schema.json`.
- Install profile closure: every profile in `ops/k8s/install-matrix.json` must pass `helm lint`, `helm template`, and `kubeconform`.
- Docs output closure: `mkdocs build --strict` must publish to the configured `site_dir`, and that output must contain `index.html` plus assets.
- If this closure fails, reproduce with `bijux dev atlas docs site-dir --format json` before changing the workflow or `mkdocs.yml`.

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
