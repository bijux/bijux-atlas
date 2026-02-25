# Contract

- Area: `ops/observe`
- schema_version: `1`
- Canonical parent contract: `ops/CONTRACT.md`

## Authored vs Generated

| Path | Role |
| --- | --- |
| `ops/observe/alert-catalog.json` | Authored alert catalog |
| `ops/observe/slo-definitions.json` | Authored SLO definitions |
| `ops/observe/telemetry-drills.json` | Authored telemetry drill catalog |
| `ops/observe/readiness.json` | Authored observability readiness policy |
| `ops/observe/suites/suites.json` | Authored observability suite definitions |
| `ops/observe/generated/telemetry-index.json` | Generated observability index |

## Schema References

| Artifact | Schema |
| --- | --- |
| `ops/observe/alert-catalog.json` | `ops/schema/observe/alert-catalog.schema.json` |
| `ops/observe/slo-definitions.json` | `ops/schema/observe/slo-definitions.schema.json` |
| `ops/observe/telemetry-drills.json` | `ops/schema/observe/telemetry-drills.schema.json` |
| `ops/observe/readiness.json` | `ops/schema/observe/readiness.schema.json` |
| `ops/observe/suites/suites.json` | `ops/schema/observe/suites.schema.json` |
| `ops/observe/generated/telemetry-index.json` | `ops/schema/observe/telemetry-index.schema.json` |

## Invariants

- Canonical naming is `observe`; legacy `obs` names and paths are forbidden in authored artifacts.
- Public telemetry surface is the union of published metrics, alerts, traces, logs fields, and dashboard contracts under `ops/observe/contracts/`.
- Alert catalog coverage must reference SLOs and stable severity semantics.
- SLO definitions and telemetry drills are authored policy; generated telemetry index is derived evidence.
- Observability suites must be deterministic for the same drill catalog and contracts.
- Generated telemetry index is immutable evidence output and must not be hand-edited.
