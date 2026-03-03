# Check Reports

- Owner: `docs-governance`
- Type: `reference`
- Audience: `contributor`
- Stability: `stable`
- Reason to exist: list the governed check report schemas emitted by the checks suite.

## Schema Index

- SSOT index: `configs/contracts/reports/checks/schema-index.json`
- Schema root: `configs/contracts/reports/checks/`

## Governed check report ids

| Report ID | Schema | Typical producers |
| --- | --- | --- |
| `check-rustfmt` | `configs/contracts/reports/checks/check-rustfmt.schema.json` | `CHECK-RUSTFMT-001` |
| `check-clippy` | `configs/contracts/reports/checks/check-clippy.schema.json` | `CHECK-RUST-CLIPPY-001` |
| `check-config-format` | `configs/contracts/reports/checks/check-config-format.schema.json` | `CHECK-LINT-POLICY-001`, `CHECK-CONFIGS-LINT-001` |
| `check-docs-links` | `configs/contracts/reports/checks/check-docs-links.schema.json` | `CHECK-DOCS-VALIDATE-001` |
| `check-helm-lint` | `configs/contracts/reports/checks/check-helm-lint.schema.json` | `CHECK-K8S-VALIDATE-001` |
| `check-kubeconform` | `configs/contracts/reports/checks/check-kubeconform.schema.json` | `CHECK-K8S-VALIDATE-001` |
| `check-deps` | `configs/contracts/reports/checks/check-deps.schema.json` | `CHECK-SUPPLY-CHAIN-DENY-001`, `CHECK-SUPPLY-CHAIN-AUDIT-001` |
| `check-suite-summary` | `configs/contracts/reports/checks/check-suite-summary.schema.json` | control-plane suite wrappers and contract-backed required lanes |

## Reading order

1. Open the schema for the report id.
2. Open the emitted report under `artifacts/suites/<suite>/<run_id>/<check_id>/`.
3. Use the paired `result.json` and `suite-summary.json` files for execution status and lane-level context.
