# Policy Enforcement Map

## Purpose
Maps each repository policy to the exact enforcement point so policy drift is observable and reviewable.

## Policy To Enforcement
| Policy | Enforcement location(s) |
|---|---|
| Root/layout contract | `bijux dev atlas check root-shape`, `make check-gates`, `ci-root-layout` |
| Symlink policy | `crates/bijux-dev-atlas/src/checks/layout/policies/root/check_symlink_policy.py`, `docs/development/symlinks.md`, `bijux dev atlas-check-layout` |
| Makefile-only workflow execution | `crates/bijux-dev-atlas/src/checks/layout/workflows/check_workflows_make_only.py`, `ci-workflows-make-only` |
| SSOT contract drift | `make ssot-check`, `api-contract-check`, `ci-api-contract` |
| OpenAPI drift + breaking change detection | `bijux dev atlas check run --suite ci_fast breakage`, `bijux dev atlas contracts generate --generators openapi artifacts`, `ci-openapi-drift` |
| Docs drift and link integrity | `make docs`, `make docs-freeze`, `bijux dev atlas docs link-check --report text`, `ci-docs-build` |
| Shell surface governance | `bijux dev atlas docs script-headers-check`, `bijux dev atlas check run --group make --id checks_make_public_target_bijux dev atlas_mapping`, `bijux dev atlas check run --group repo --id checks_repo_command_scripts_registry`, `bijux dev atlas check run --group repo --id checks_repo_no_direct_script_runs` |
| Policy schema drift | `bijux dev atlas policies schema-drift`, `ci-policy-schema-drift` |
| Policy relaxations registry (SSOT exceptions) | `configs/policy/policy-relaxations.json`, `bijux dev atlas policies scan-rust-relaxations`, `bijux dev atlas policies check --fail-fast`, `ci-policy-relaxations` |
| Policy enforcement coverage contract | `configs/policy/policy-enforcement-coverage.json`, `bijux dev atlas policies enforcement-status --enforce`, `docs/_generated/policy-enforcement-status.md`, `ci-policy-enforcement` |
| Escape-hatch env control (`ALLOW_*`) | `configs/ops/env.schema.json`, `bijux dev atlas policies allow-env-lint`, `ci-policy-allow-env` |
| Ops policy reflection | `bijux dev atlas ops policy-audit`, `ci-ops-policy-audit` |
| Crate boundary guardrails | `crates/bijux-atlas-core/tests/guardrails.rs`, `make architecture-check` |
| SQLite index/schema contracts | `bin/bijux-atlas contracts check --checks sqlite-indexes`, ingest schema/index drift tests, `ci-sqlite-*` |

## Relaxation Rules
- Exception entries are SSOT in `configs/policy/policy-relaxations.json`.
- Every enforced relaxation marker in code must include an `ATLAS-EXC-XXXX` tag.
- Expired exceptions fail CI.
- Wildcard exception scopes are forbidden.
- Exception budgets are enforced per policy.
