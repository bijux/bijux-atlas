# Contract

- Area: `ops/report`
- schema_version: `1`
- Canonical parent contract: `ops/CONTRACT.md`

## Authored vs Generated

| Path | Role |
| --- | --- |
| `ops/report/schema.json` | Authored unified report contract |
| `ops/report/evidence-levels.json` | Authored evidence-level policy |
| `ops/report/examples/unified-report-example.json` | Authored curated report example |
| `ops/report/generated/report-diff.json` | Generated report diff |
| `ops/report/generated/historical-comparison.json` | Generated historical comparison |
| `ops/report/generated/release-evidence-bundle.json` | Generated release evidence bundle |
| `ops/report/generated/readiness-score.json` | Generated readiness score |

## Schema References

| Artifact | Schema |
| --- | --- |
| `ops/report/schema.json` | `ops/schema/report/schema.json` |
| `ops/report/evidence-levels.json` | `ops/schema/report/evidence-levels.schema.json` |
| `ops/report/generated/report-diff.json` | `ops/schema/report/report-diff.schema.json` |
| `ops/report/generated/historical-comparison.json` | `ops/schema/report/historical-comparison.schema.json` |
| `ops/report/generated/release-evidence-bundle.json` | `ops/schema/report/release-evidence-bundle.schema.json` |
| `ops/report/generated/readiness-score.json` | `ops/schema/report/readiness-score.schema.json` |

## Invariants

- Unified report schema and evidence-level policy are authored truth for report assembly.
- Generated report artifacts are deterministic for the same inventory, schema index, and gate outputs.
- Evidence bundle must contain hashes for required SSOT artifacts and schema index inputs.
- Report diff and historical comparison must be based on pinned artifact snapshots.
- Readiness score calculation rules are stable and versioned with report schemas.
- Report docs under `ops/report/docs/` define contract semantics and must remain linked from the reference index.
