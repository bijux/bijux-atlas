---
title: Start the Server
audience: mixed
type: how-to
status: canonical
owner: atlas-docs
last_reviewed: 2026-03-15
---

# Start the Server

Once you have published and promoted a sample dataset into a serving store,
starting the local server is straightforward: point the runtime at that store
root and keep the cache root under `artifacts/`.

The important precondition is real: the sample dataset must already be
published and catalog-promoted. Starting the server against a build root is a
workflow mistake, not a supported shortcut.

## Runtime Inputs

```mermaid
flowchart LR
    BuildRoot[artifacts/getting-started/tiny-build] --> Publish[dataset publish]
    Publish --> Store[artifacts/getting-started/tiny-store]
    Store --> Server[bijux-atlas-server]
    Cache[artifacts/getting-started/server-cache] --> Server
    Config[flags or config file] --> Server
```

This runtime-input diagram shows why the server startup page comes after
publication. Atlas expects the runtime to start from serving store state, plus
explicit cache and configuration inputs.

## Validate the Runtime Shape First

```bash
mkdir -p artifacts/getting-started/server-cache

cargo run -p bijux-atlas --bin bijux-atlas-server -- \
  --store-root artifacts/getting-started/tiny-store \
  --cache-root artifacts/getting-started/server-cache \
  --validate-config
```

This is a low-risk first step because it validates runtime inputs without
committing to a long-running process.

If `--validate-config` fails, fix that before trying to bind the server. A broken validation step usually means startup would fail or produce misleading partial behavior anyway.

## Start the Local Server

```bash
cargo run -p bijux-atlas --bin bijux-atlas-server -- \
  --bind 127.0.0.1:8080 \
  --store-root artifacts/getting-started/tiny-store \
  --cache-root artifacts/getting-started/server-cache
```

Leave the server running in one terminal while you query it from another.

## Startup Sequence

```mermaid
sequenceDiagram
    participant User
    participant Server
    participant Store
    participant Cache
    User->>Server: start with bind, store-root, cache-root
    Server->>Store: open store root
    Server->>Cache: prepare cache directory
    Server-->>User: bind and accept requests
```

This startup sequence keeps the server’s dependencies visible. If startup
fails, the first place to look is the store root, cache root, or resolved
configuration, not the query layer.

## First Health Checks

In another terminal:

```bash
curl -s http://127.0.0.1:8080/healthz
curl -s http://127.0.0.1:8080/readyz
curl -s http://127.0.0.1:8080/v1/version
```

Interpret them in order:

- `healthz` tells you whether the process is alive enough to answer
- `readyz` tells you whether it considers itself ready to serve
- `/v1/version` tells you whether the API surface is reachable beyond pure health plumbing

## What the Startup Model Is Protecting

```mermaid
flowchart TD
    Validate[Validate runtime inputs] --> Bind[Bind server]
    Bind --> Health[Expose health endpoints]
    Health --> Query[Serve query endpoints]
```

This model shows the order Atlas is trying to enforce. Readiness should come
after configuration and artifact resolution, not before.

Atlas tries to make startup failure explicit rather than turning configuration drift into partial runtime behavior.

## If the Server Does Not Start

- confirm the sample dataset was built, published, and catalog-promoted first
- confirm the `--store-root` points at the serving store, not the ingest build root or source fixture directory
- confirm the `--cache-root` is under `artifacts/` and writable
- re-run with `--print-effective-config` if you need to inspect resolved settings
- treat “process started” and “runtime is ready” as different questions

## What a Good Startup Proves

- the server can resolve the store root and cache root you supplied
- the runtime can bind and expose health endpoints
- you are ready to move from startup checks to real query checks

## Reading Rule

Use this page when the serving store is ready and the remaining question is how
to start the runtime without blurring build state and serving state.
