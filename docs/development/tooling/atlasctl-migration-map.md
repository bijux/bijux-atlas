# Atlasctl Migration Map

## Purpose
Map legacy atlasctl check surfaces to `bijux-dev-atlas` check identifiers and routes.

## Command Route
- Legacy: `./bin/atlasctl check ...`
- Replacement: `bijux dev atlas run ...`

## Check Mapping
| Legacy area | Replacement suite/domain | Replacement check IDs |
| --- | --- | --- |
| ops surface manifest | `--suite ops_fast` | `ops_surface_manifest` |
| ops required contracts | `--suite ops_fast` | `ops_tree_contract` |
| ops legacy tooling refs | `--suite ops_fast` | `ops_no_legacy_tooling_refs` |
| ops generated mirror policy | `--suite ops_fast` | `ops_generated_readonly_markers` |
| ops schema presence | `--suite ops_fast` | `ops_schema_presence` |
| ops inventory json integrity | `--suite ops_fast` | `ops_manifest_integrity` |
| ops index inventory | `--suite ops_fast` | `ops_surface_inventory` |
| ops evidence not committed | `--suite ops_fast` | `ops_artifacts_not_tracked` |
| dev-atlas no python legacy refs | `--suite ops_fast` | `ops_no_python_legacy_runtime_refs` |
| dev-atlas no legacy runner paths | `--suite ops_fast` | `ops_no_legacy_runner_paths` |

## Canonical Governance Entry
- `bijux dev atlas doctor`
