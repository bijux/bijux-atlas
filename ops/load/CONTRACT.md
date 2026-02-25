# Contract

- Area: `ops/load`
- schema_version: `1`
- Canonical parent contract: `ops/CONTRACT.md`

## Authored vs Generated

| Path | Role |
| --- | --- |
| `ops/load/suites/suites.json` | Authored suite catalog SSOT |
| `ops/load/thresholds/*.thresholds.json` | Authored thresholds SSOT |
| `ops/load/scenarios/*.json` | Authored scenarios SSOT |
| `ops/load/k6/suites/*.js` | Authored suite scripts |
| `ops/load/generated/suites.manifest.json` | Generated suite manifest |
| `ops/load/generated/load-summary.json` | Generated summary |
| `ops/load/generated/load-drift-report.json` | Generated drift report |

## Schema References

| Artifact | Schema |
| --- | --- |
| `ops/load/suites/suites.json` | `ops/schema/load/suite-manifest.schema.json` |
| `ops/load/thresholds/*.thresholds.json` | `ops/schema/load/thresholds.schema.json` |
| `ops/load/contracts/deterministic-seed-policy.json` | `ops/schema/load/deterministic-seed-policy.schema.json` |
| `ops/load/generated/suites.manifest.json` | `ops/schema/load/suite-manifest.schema.json` |
| `ops/load/generated/load-summary.json` | `ops/schema/load/load-summary.schema.json` |
| `ops/load/generated/load-drift-report.json` | `ops/schema/load/load-drift-report.schema.json` |

## Invariants

- No duplicate authored truth is allowed; suite SSOT is `ops/load/suites/suites.json` and threshold SSOT is `ops/load/thresholds/*.thresholds.json`.
- Schema references for this domain must resolve only to `ops/schema/**`.
- Behavior source outside canonical k6 script surfaces is forbidden under `ops/load`.
- The semantic domain name `obs` is prohibited; only canonical `observe` naming is valid.
- Generated load artifacts must include `generated_by` and `schema_version` metadata.
- Load docs must be linked from `ops/load/INDEX.md`; orphan docs are forbidden.
- Authored JSON in `ops/load/k6/manifests/` is forbidden.
- Generated load summary and drift reports must be deterministic for identical authored inputs.

## Enforcement Links

- `checks_ops_required_files_contracts`
- `checks_ops_domain_contract_structure`
