# Contract

- Area: `ops/observe`
- schema_version: `1`
- contract_version: `1.0.0`
- contract_taxonomy: `behavioral`
- Canonical parent contract: `ops/CONTRACT.md`

## Authored vs Generated

| Path | Role |
| --- | --- |
| `ops/observe/alert-catalog.json` | Authored alert catalog |
| `ops/observe/slo-definitions.json` | Authored SLO catalog |
| `ops/observe/telemetry-drills.json` | Authored drill catalog |
| `ops/observe/readiness.json` | Authored readiness policy |
| `ops/observe/suites/suites.json` | Authored suite catalog |
| `ops/observe/generated/telemetry-index.json` | Generated telemetry index |

## Schema References

| Artifact | Schema |
| --- | --- |
| `ops/observe/alert-catalog.json` | `ops/schema/observe/alert-catalog.schema.json` |
| `ops/observe/slo-definitions.json` | `ops/schema/observe/slo-definitions.schema.json` |
| `ops/observe/telemetry-drills.json` | `ops/schema/observe/telemetry-drills.schema.json` |
| `ops/observe/readiness.json` | `ops/schema/observe/readiness.schema.json` |
| `ops/observe/suites/suites.json` | `ops/schema/observe/suites.schema.json` |
| `ops/observe/generated/telemetry-index.json` | `ops/schema/observe/telemetry-index.schema.json` |

## Contract Taxonomy

- Structural contract: authored observability catalogs and suite definitions define stable telemetry surfaces.
- Behavioral contract: drill/readiness/SLO artifacts define expected operational behavior under incidents and release checks.

## Invariants

- No duplicate authored truth is allowed; authored observability policy lives only in `ops/observe/*.json` and `ops/observe/suites/*.json`.
- Schema references for this domain must resolve only to `ops/schema/**`.
- Behavior source is forbidden in `ops/observe`; runtime execution logic remains outside `ops/`.
- The semantic domain name `obs` is prohibited; only canonical `observe` naming is valid.
- Generated observe artifacts must include `generated_by` and `schema_version` metadata.
- Observe docs must be linked from `ops/observe/INDEX.md`; orphan docs are forbidden.
- Public telemetry surface is defined by contracts in `ops/observe/contracts/` and must not drift silently.
- Generated telemetry index output must be deterministic for identical authored inputs.

## Runtime Evidence Mapping

- Telemetry index evidence: `ops/observe/generated/telemetry-index.json`
- Curated docs-link evidence: `ops/_generated.example/docs-links-report.json`
- Curated schema coverage evidence: `ops/_generated.example/schema-coverage-report.json`

## Enforcement Links

- `checks_ops_no_scripts_areas_or_xtask_refs`
- `checks_ops_required_files_contracts`
- `checks_ops_domain_contract_structure`
