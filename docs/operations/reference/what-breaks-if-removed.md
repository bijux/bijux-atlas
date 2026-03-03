# What Breaks If Removed Reference

- Owner: `bijux-atlas-operations`
- Tier: `generated`
- Audience: `operators`
- Source-of-truth: `ops/_generated.example/what-breaks-if-removed-report.json`

## Removal Impact Targets

| Path | Impact | Consumers |
| --- | --- | --- |
| `ops/inventory/contracts-map.json` | `breaking` | `checks_ops_inventory_contract_integrity, checks_ops_file_usage_and_orphan_contract` |
| `ops/load/scenarios` | `warning` | `checks_ops_minimalism_and_deletion_safety` |
