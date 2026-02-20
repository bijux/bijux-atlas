# Policy Enforcement Map

## Purpose
Maps each repository policy to the exact enforcement point so policy drift is observable and reviewable.

## Policy To Enforcement
| Policy | Enforcement location(s) |
|---|---|
| Root/layout contract | `scripts/areas/layout/check_root_shape.sh`, `make layout-check`, `ci-root-layout` |
| Symlink policy | `scripts/areas/layout/check_symlink_policy.py`, `docs/development/symlinks.md`, `layout-check` |
| Makefile-only workflow execution | `scripts/areas/layout/check_workflows_make_only.py`, `ci-workflows-make-only` |
| SSOT contract drift | `make ssot-check`, `api-contract-check`, `ci-api-contract` |
| OpenAPI drift + breaking change detection | `scripts/areas/public/openapi-diff-check.sh`, `bin/bijux-atlas contracts check --checks breakage`, `ci-openapi-drift` |
| Docs drift and link integrity | `make docs`, `make docs-freeze`, `scripts/areas/public/check-markdown-links.sh`, `ci-docs-build` |
| Script surface governance | `scripts/areas/docs/check_script_headers.py`, `scripts/areas/layout/check_make_public_scripts.py`, `scripts/areas/layout/check_scripts_buckets.py`, `scripts-audit` |
| Policy schema drift | `scripts/areas/public/policy-schema-drift.py`, `ci-policy-schema-drift` |
| Policy relaxations registry (SSOT exceptions) | `configs/policy/policy-relaxations.json`, `scripts/areas/policy/find_relaxations.sh`, `bijux-atlas policies scan-rust-relaxations`, `scripts/areas/public/policy-audit.py`, `ci-policy-relaxations` |
| Policy enforcement coverage contract | `configs/policy/policy-enforcement-coverage.json`, `scripts/areas/public/policy-enforcement-status.py`, `docs/_generated/policy-enforcement-status.md`, `ci-policy-enforcement` |
| Escape-hatch env control (`ALLOW_*`) | `configs/ops/env.schema.json`, `scripts/areas/public/check-allow-env-schema.py`, `ci-policy-allow-env` |
| Ops policy reflection | `scripts/areas/public/ops-policy-audit.py`, `ci-ops-policy-audit` |
| Crate boundary guardrails | `crates/bijux-atlas-core/tests/guardrails.rs`, `make architecture-check` |
| SQLite index/schema contracts | `bin/bijux-atlas contracts check --checks sqlite-indexes`, ingest schema/index drift tests, `ci-sqlite-*` |

## Relaxation Rules
- Exception entries are SSOT in `configs/policy/policy-relaxations.json`.
- Every enforced relaxation marker in code must include an `ATLAS-EXC-XXXX` tag.
- Expired exceptions fail CI.
- Wildcard exception scopes are forbidden.
- Exception budgets are enforced per policy.
