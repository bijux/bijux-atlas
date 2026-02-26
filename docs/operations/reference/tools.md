# Tools Reference

- Owner: `bijux-atlas-operations`
- Tier: `generated`
- Audience: `operators`
- Source-of-truth: `ops/inventory/tools.toml`

## Tools

| Tool | Required | Probe Args | Version Regex |
| --- | --- | --- | --- |
| `helm` | `true` | `version --short` | `(\d+\.\d+\.\d+)` |
| `k6` | `false` | `version` | `(\d+\.\d+\.\d+)` |
| `kind` | `true` | `--version` | `(\d+\.\d+\.\d+)` |
| `kubectl` | `true` | `version --client --short` | `(\d+\.\d+\.\d+)` |

## Regenerate

- `python3 scripts/docs/generate_operations_references.py --write`
