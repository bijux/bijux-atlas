# Atlas Check Migration Map

This map tracks the migration from `atlasctl` checks to `bijux-dev-atlas`.

| atlasctl check | bijux-dev-atlas check | status |
| --- | --- | --- |
| `check_ops_surface_manifest` | `ops_surface_manifest` | migrated |
| `check_ops_*` contract family | `ops_tree_contract` | started |
| `check_ops_*` generated policy family | `ops_generated_readonly_markers` | started |
| `check_ops_*` schema presence family | `ops_schema_presence` | started |
| `check_ops_*` inventory integrity family | `ops_manifest_integrity` | started |
| `check_ops_*` inventory index family | `ops_surface_inventory` | started |
| `check_ops_*` artifacts evidence family | `ops_artifacts_not_tracked` | started |
| `check_ops_*` legacy tooling refs family | `ops_no_legacy_tooling_refs` | started |
