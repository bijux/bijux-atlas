# Component Responsibilities

- Owner: `atlas-platform`
- Stability: `stable`

## What Belongs Where

- Ingest (`bijux-atlas-ingest`): parse/normalize source inputs, produce deterministic artifacts, emit QC.
- Serve (`bijux-atlas-server` + `bijux-atlas-query` + `bijux-atlas-api`): request validation, query execution, response envelopes, cache semantics.
- Store (`bijux-atlas-store` + server store adapters): catalog/manifests/artifact fetch and integrity surface.
- Ops (`ops/`, make `ops-*`): cluster stack lifecycle, deploy, smoke/load/observability validation.

## Non-goals

- Serve path does not perform ingest transformations.
- Ingest path does not expose HTTP runtime behavior.
- Ops scripts are orchestration wrappers, not business logic SSOT.

## Effect Boundaries

- Query: CPU + SQLite query planning/execution only.
- Server/cache/store adapters: fs/net/clock side effects.
- Ingest: fs/parse/transform/build side effects.
