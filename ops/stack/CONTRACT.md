# Contract

- Area: `ops/stack`
- schema_version: `1`
- contract_version: `1.0.0`
- contract_taxonomy: `hybrid`
- Canonical parent contract: `ops/CONTRACT.md`

## Authored vs Generated

| Path | Role |
| --- | --- |
| `ops/inventory/pins.yaml` | Authored pins SSOT |
| `ops/stack/profiles.json` | Authored stack profile catalog |
| `ops/stack/stack.toml` | Authored stack composition contract |
| `ops/stack/service-dependency-contract.json` | Authored service dependency and health contract |
| `ops/stack/generated/version-manifest.json` | Generated image/version manifest |
| `ops/stack/generated/stack-index.json` | Generated stack index |
| `ops/stack/generated/dependency-graph.json` | Generated dependency graph |
| `ops/stack/generated/artifact-metadata.json` | Generated artifact metadata |
| `ops/stack/generated/versions.json` | Generated version summary |

## Schema References

| Artifact | Schema |
| --- | --- |
| `ops/stack/profiles.json` | `ops/schema/stack/profile-manifest.schema.json` |
| `ops/stack/service-dependency-contract.json` | `ops/schema/stack/service-dependency-contract.schema.json` |
| `ops/stack/generated/version-manifest.json` | `ops/schema/stack/version-manifest.schema.json` |
| `ops/stack/generated/dependency-graph.json` | `ops/schema/stack/dependency-graph.schema.json` |
| `ops/stack/generated/artifact-metadata.json` | `ops/schema/stack/artifact-metadata.schema.json` |
| `ops/stack/generated/versions.json` | `ops/schema/stack/versions.schema.json` |
| `ops/inventory/pins.yaml` | `ops/schema/inventory/pins.schema.json` |

## Contract Taxonomy

- Structural contract: stack composition and profile metadata define stable deployment surfaces.
- Behavioral contract: generated version/dependency outputs and pin-freeze enforcement define runtime rollout behavior.

## Invariants

- No duplicate authored truth is allowed; pin SSOT is `ops/inventory/pins.yaml`.
- Schema references for this domain must resolve only to `ops/schema/**`.
- Behavior source is forbidden in `ops/stack`; stack behavior executes outside `ops/`.
- The semantic domain name `obs` is prohibited; only canonical `observe` naming is valid.
- Generated stack artifacts must include `generated_by` and `schema_version` metadata.
- Stack docs must be linked from `ops/stack/INDEX.md`; orphan docs are forbidden.
- Pin registries and generated stack manifests must be deterministic for identical authored inputs.
- Pin lifecycle compliance with `ops/inventory/pin-freeze.json` is mandatory for release readiness.
- Service dependency contract coverage is mandatory: required services must appear in their declared stack profiles.

## Runtime Evidence Mapping

- Stack version evidence: `ops/stack/generated/version-manifest.json`
- Dependency evidence: `ops/stack/generated/dependency-graph.json`
- Stack health evidence: `ops/report/generated/stack-health-report.json`

## Enforcement Links

- `checks_ops_inventory_contract_integrity`
- `checks_ops_required_files_contracts`
- `checks_ops_domain_contract_structure`
