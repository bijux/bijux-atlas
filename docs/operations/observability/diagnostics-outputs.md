# Diagnostics Outputs

Diagnostics endpoints emit machine-readable runtime snapshots.

Primary outputs:

- `/debug/diagnostics`
- `/debug/runtime-stats`
- `/debug/system-info`
- `/debug/build-metadata`
- `/debug/runtime-config`
- `/debug/dataset-registry`
- `/debug/shard-map`
- `/debug/query-planner-stats`
- `/debug/cache-stats`

Collection path:

- `bijux-dev-atlas system debug ...`
- artifacts are stored under `artifacts/system/diagnostics/`
