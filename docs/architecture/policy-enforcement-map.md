# Policy Enforcement Map

## Purpose
Maps each repository policy to the exact enforcement point so policy drift is observable and reviewable.

## Policy To Enforcement
| Policy | Enforcement location(s) |
|---|---|
| Root/layout contract | `packages/bijux-atlas-scripts/src/bijux_atlas_scripts/layout_checks/check_root_shape.sh`, `make layout-check`, `ci-root-layout` |
| Symlink policy | `packages/bijux-atlas-scripts/src/bijux_atlas_scripts/layout_checks/check_symlink_policy.py`, `docs/development/symlinks.md`, `layout-check` |
| Makefile-only workflow execution | `packages/bijux-atlas-scripts/src/bijux_atlas_scripts/layout_checks/check_workflows_make_only.py`, `ci-workflows-make-only` |
| SSOT contract drift | `make ssot-check`, `api-contract-check`, `ci-api-contract` |
| OpenAPI drift + breaking change detection | `atlasctl contracts check --checks drift breakage`, `atlasctl contracts generate --generators openapi artifacts`, `ci-openapi-drift` |
| Docs drift and link integrity | `make docs`, `make docs-freeze`, `atlasctl docs link-check --report text`, `ci-docs-build` |
| Script surface governance | `atlasctl docs script-headers-check`, `packages/bijux-atlas-scripts/src/bijux_atlas_scripts/layout_checks/check_make_public_scripts.py`, `packages/bijux-atlas-scripts/src/bijux_atlas_scripts/layout_checks/check_scripts_buckets.py`, `scripts-audit` |
| Policy schema drift | `atlasctl policies schema-drift`, `ci-policy-schema-drift` |
| Policy relaxations registry (SSOT exceptions) | `configs/policy/policy-relaxations.json`, `atlasctl policies scan-rust-relaxations`, `atlasctl policies check --fail-fast`, `ci-policy-relaxations` |
| Policy enforcement coverage contract | `configs/policy/policy-enforcement-coverage.json`, `atlasctl policies enforcement-status --enforce`, `docs/_generated/policy-enforcement-status.md`, `ci-policy-enforcement` |
| Escape-hatch env control (`ALLOW_*`) | `configs/ops/env.schema.json`, `atlasctl policies allow-env-lint`, `ci-policy-allow-env` |
| Ops policy reflection | `atlasctl ops policy-audit`, `ci-ops-policy-audit` |
| Crate boundary guardrails | `crates/bijux-atlas-core/tests/guardrails.rs`, `make architecture-check` |
| SQLite index/schema contracts | `bin/bijux-atlas contracts check --checks sqlite-indexes`, ingest schema/index drift tests, `ci-sqlite-*` |

## Relaxation Rules
- Exception entries are SSOT in `configs/policy/policy-relaxations.json`.
- Every enforced relaxation marker in code must include an `ATLAS-EXC-XXXX` tag.
- Expired exceptions fail CI.
- Wildcard exception scopes are forbidden.
- Exception budgets are enforced per policy.
