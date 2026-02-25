# bijux-dev-atlas-policies

![Version](https://img.shields.io/badge/version-0.1.0-informational.svg) ![License: Apache-2.0](https://img.shields.io/badge/license-Apache%202.0-blue.svg) ![Docs](https://img.shields.io/badge/docs-contract-stable-brightgreen.svg)

Development control-plane policy contracts and pure policy evaluation logic.

## Scope
Belongs here:
- Repository shape policies used by `bijux dev atlas`.
- Ops/check/config/dev governance contracts for local and CI control-plane checks.
- Relaxations and ratchets for dev control-plane policy rollout.

Belongs in `bijux-atlas-policies`:
- Runtime product policy contracts for atlas data/query/store/server surfaces.

## Stability
- Policy ids in `POLICY_REGISTRY` are machine-stable.
- Policy evaluation consumes pure snapshots and emits deterministic violations.
- Relaxations require explicit expiry dates.

## Workflow
- Edit policy source: `ops/inventory/policies/dev-atlas-policy.json`.
- Keep schema synchronized: `ops/inventory/policies/dev-atlas-policy.schema.json`.
- Run: `cargo test -p bijux-dev-atlas-policies`.
