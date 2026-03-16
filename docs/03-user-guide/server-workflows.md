---
title: Server Workflows
audience: user
type: guide
status: canonical
owner: atlas-docs
last_reviewed: 2026-03-15
---

# Server Workflows

Server workflows cover the product-facing runtime surface: starting the server, checking health, and using the main HTTP routes as intended.

This page is about normal runtime usage after a valid serving store exists. It is not the operator runbook for deployment, scaling, or incident response.

## Server Workflow Model

```mermaid
flowchart TD
    Store[Serving store] --> Start[Start server]
    Start --> Health[Health and readiness]
    Health --> Query[Query traffic]
    Query --> Observe[Metrics and diagnostics]
```

## Main Server Surfaces

```mermaid
flowchart LR
    Runtime[Server runtime] --> Health[Health and readiness routes]
    Runtime --> Metrics[Metrics route]
    Runtime --> Version[Version route]
    Runtime --> Data[Dataset and query routes]
    Runtime --> OpenAPI[OpenAPI route]
```

Not every surface has the same audience:

- `/healthz`, `/readyz`, and `/metrics` are primarily operational surfaces
- `/v1/version`, `/v1/datasets`, and query routes are product-facing runtime surfaces
- `/v1/openapi.json` is a contract and integration surface

## Common Day-to-Day Actions

- validate config before startup
- bind to a local or service address
- check health and readiness before sending traffic
- verify dataset discovery through `/v1/datasets`
- confirm API identity through `/v1/version`
- use metrics and OpenAPI deliberately rather than as substitutes for actual query validation

## Practical Startup

```bash
cargo run -p bijux-atlas --bin bijux-atlas-server -- \
  --bind 127.0.0.1:8080 \
  --store-root artifacts/getting-started/tiny-store \
  --cache-root artifacts/getting-started/server-cache
```

## Important Everyday Checks

```bash
curl -s http://127.0.0.1:8080/healthz
curl -s http://127.0.0.1:8080/readyz
curl -s http://127.0.0.1:8080/metrics
curl -s http://127.0.0.1:8080/v1/openapi.json
```

Those checks answer different questions. A healthy metrics endpoint does not prove that the expected
dataset is published. A reachable OpenAPI document does not prove that an environment is
production-ready.

## Operational Boundary

This page explains normal usage of the runtime surface. For deployment, rollback, resource tuning, and incident handling, move to [Operations](../04-operations/index.md).

## Purpose

This page explains the Atlas material for server workflows and points readers to the canonical checked-in workflow or boundary for this topic.

## Stability

This page is part of the canonical Atlas docs spine. Keep it aligned with the current repository behavior and adjacent contract pages.
