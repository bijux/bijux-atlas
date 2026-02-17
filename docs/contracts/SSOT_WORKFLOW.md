# How SSOT Works In Atlas

- Owner: `docs-governance`

Atlas uses `docs/contracts/` as single-source-of-truth for machine contracts.

## Contracts
- `ERROR_CODES.json` -> generated Rust constants + OpenAPI enum validation.
- `METRICS.json` -> generated observability collector contract.
- `TRACE_SPANS.json` -> generated trace span name/attributes contract.
- `ENDPOINTS.json` -> route registry cross-check against server and OpenAPI.
- `CHART_VALUES.json` -> chart values key contract.
- `CLI_COMMANDS.json` -> CLI command list + docs consistency.
- `CONFIG_KEYS.json` -> config/env key allowlist contract.
- `ARTIFACT_SCHEMA.json` -> artifact manifest/QC/db-meta schema contract.
- `POLICY_SCHEMA.json` -> policy schema contract mirrored from `configs/policy/policy.schema.json`.

## Enforcement
- Local: `make ssot-check`
- CI: `ssot-drift` job in `.github/workflows/ci.yml`
- Policy lint includes SSOT gate.
- Compatibility guard: `scripts/contracts/check_breaking_contract_change.py` compares against latest `v*` tag.

## No Manual Drift
Generated files under `docs/_generated/contracts/`, `crates/bijux-atlas-api/src/generated/`, `crates/bijux-atlas-server/src/telemetry/generated/`, and `observability/metrics_contract.json` must not be hand-edited.

## What

Defines a stable contract surface for this topic.

## Why

Prevents ambiguity and drift across CLI, API, and operations.

## Scope

Applies to atlas contract consumers and producers.

## Non-goals

Does not define internal implementation details beyond the contract surface.

## Failure modes

Invalid contract input is rejected with stable machine-readable errors.

## Examples

```bash
$ make ssot-check
```

Expected output: a zero exit code and "contract artifacts generated" for successful checks.

## How to verify

Run `make docs docs-freeze ssot-check` and confirm all commands exit with status 0.

## See also

- [Contracts Overview](README.md)
- [SSOT Workflow](SSOT_WORKFLOW.md)
- [Terms Glossary](../_style/TERMS_GLOSSARY.md)
