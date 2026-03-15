---
title: Runtime Configuration
audience: operator
type: guide
status: canonical
owner: atlas-docs
last_reviewed: 2026-03-15
---

# Runtime Configuration

Runtime configuration controls how Atlas serves, limits, caches, logs, and responds. It does not redefine the content of published artifacts.

## Runtime Config Boundary

```mermaid
flowchart LR
    Artifacts[Published artifacts] --> Serve[Serving state]
    RuntimeConfig[Runtime config] --> Serve
    RuntimeConfig --> Limits[Rate, concurrency, and cache policy]
```

## The Main Rule

Do not mix runtime configuration with release content configuration.

- published artifacts define what data exists
- runtime config defines how the server behaves around that data

## Configuration Inputs

```mermaid
flowchart TD
    Flags[Startup flags] --> Runtime[Runtime configuration]
    File[Config file] --> Runtime
    Env[Environment variables] --> Runtime
    Runtime --> Validation[Validation and effective config]
```

## Operational Practices

- validate config before rollout when possible
- prefer explicit paths and values over environment-dependent assumptions
- keep cache roots and artifact roots clearly separated
- inspect effective config when behavior is surprising

## Example Runtime Validation

```bash
cargo run -p bijux-atlas --bin bijux-atlas-server -- \
  --store-root artifacts/getting-started/tiny-store \
  --cache-root artifacts/getting-started/server-cache \
  --validate-config
```

## Runtime Config Questions to Ask

- where is the serving store root?
- where does the cache live?
- what are the active runtime limits?
- what logging and telemetry sinks are active?
- how will readiness and overload behave under stress?

