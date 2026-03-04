# Runtime Diagnostics

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Reason to exist: define the runtime diagnostics contract and the operator retrieval flow.

## Endpoints

- `/debug/diagnostics`
- `/debug/runtime-stats`
- `/debug/system-info`
- `/debug/build-metadata`
- `/debug/runtime-config`
- `/debug/dataset-registry`
- `/debug/shard-map`
- `/debug/query-planner-stats`
- `/debug/cache-stats`

## CLI retrieval

Use the system debug surface to collect deterministic reports:

```bash
bijux-dev-atlas system debug diagnostics
bijux-dev-atlas system debug runtime-state
bijux-dev-atlas system debug trace-sampling
bijux-dev-atlas system debug metrics-snapshot
bijux-dev-atlas system debug health-checks
```

Artifacts are written to `artifacts/system/diagnostics/`.
