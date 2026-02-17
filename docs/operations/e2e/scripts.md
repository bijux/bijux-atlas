# E2E Scripts

- Owner: `bijux-atlas-operations`

## What

Documents script interfaces in `e2e/scripts/`.

## Why

Keeps operational entrypoints explicit and stable.

## Scope

Bootstrapping, deploy, publish, warmup, smoke, metrics, and cleanup scripts.

## Non-goals

Does not duplicate script internals.

## Contracts

- `up.sh`, `down.sh`
- `publish_dataset.sh`, `deploy_atlas.sh`
- `warmup.sh`, `smoke_queries.sh`
- `verify_metrics.sh`, `verify_traces.sh`, `soak.sh`, `cleanup_store.sh`

## Failure modes

Undocumented script changes break local operator workflows.

## How to verify

```bash
$ ./e2e/scripts/smoke_queries.sh
$ ./e2e/scripts/verify_metrics.sh
```

Expected output: canonical smoke and metrics checks pass.

## See also

- [E2E Index](INDEX.md)
- [Development Scripts Index](../../development/scripts/INDEX.md)
- [Repo Surface](../../development/repo-surface.md)
