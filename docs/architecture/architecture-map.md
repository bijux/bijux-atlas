# Architecture Map

- Owner: `atlas-platform`
- Stability: `stable`

Generated crate-level architecture map from workspace metadata.

## Crate Nodes

| Crate | Role | Internal Dependencies |
| --- | --- | --- |
| `bijux-atlas-api` | `api-surface` | `bijux-atlas-core`, `bijux-atlas-model`, `bijux-atlas-query` |
| `bijux-atlas-cli` | `cli-ops` | `bijux-atlas-core`, `bijux-atlas-ingest`, `bijux-atlas-model`, `bijux-atlas-policies`, `bijux-atlas-query`, `bijux-atlas-store` |
| `bijux-atlas-core` | `shared-core` | `(none)` |
| `bijux-atlas-ingest` | `ingest-pipeline` | `bijux-atlas-core`, `bijux-atlas-model` |
| `bijux-atlas-model` | `shared-model` | `(none)` |
| `bijux-atlas-policies` | `policy-contracts` | `(none)` |
| `bijux-atlas-query` | `query-engine` | `bijux-atlas-core`, `bijux-atlas-model`, `bijux-atlas-policies`, `bijux-atlas-store` |
| `bijux-atlas-server` | `runtime-server` | `bijux-atlas-api`, `bijux-atlas-core`, `bijux-atlas-model`, `bijux-atlas-query`, `bijux-atlas-store` |
| `bijux-atlas-store` | `artifact-store` | `bijux-atlas-core`, `bijux-atlas-model` |

## Runtime Direction

`bijux-atlas-server -> bijux-atlas-query -> bijux-atlas-store -> immutable artifacts`

## Notes

- This file is generated; do not hand-edit.
- Regenerate via `python3 scripts/areas/docs/generate_architecture_map.py`.
