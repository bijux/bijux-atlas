# Contract

- Area: `ops/load`
- schema_version: `1`
- Canonical parent contract: `ops/CONTRACT.md`

## Authored vs Generated

| Path | Role |
| --- | --- |
| `ops/load/suites/suites.json` | Authored SSOT suite catalog |
| `ops/load/thresholds/*.thresholds.json` | Authored SSOT thresholds |
| `ops/load/scenarios/*.json` | Authored SSOT scenarios |
| `ops/load/k6/suites/*.js` | Authored suite scripts |
| `ops/load/generated/suites.manifest.json` | Generated mirror from `ops/load/suites/suites.json` |
| `ops/load/generated/load-summary.json` | Generated summary |
| `ops/load/generated/load-drift-report.json` | Generated drift report |

## Invariants

- Canonical suite manifest is `ops/load/suites/suites.json`.
- Authored JSON under `ops/load/k6/manifests/` is forbidden.
- Canonical thresholds live only under `ops/load/thresholds/`.
- Threshold filenames must be unique and follow `<suite>.thresholds.json`.
- Every k6 suite entry references an existing scenario file.
- Deterministic seed policy is defined in `ops/load/contracts/deterministic-seed-policy.json`.
- Scenario coverage in generated summary must be complete (`missing` must be empty for stable catalog).
- Threshold expectations are enforced against referenced threshold files.
