---
title: Service Topology
audience: operators
type: concept
status: canonical
owner: atlas-docs
last_reviewed: 2026-04-13
---

# Service Topology

Atlas operations span the runtime service plus supporting dependencies such as
Redis, MinIO, Prometheus, Grafana, OpenTelemetry, and Toxiproxy.

```mermaid
flowchart TD
    Client[Client] --> Runtime[Atlas runtime]
    Runtime --> Redis[Redis]
    Runtime --> Store[MinIO or store]
    Runtime --> Prom[Prometheus scrape]
    Runtime --> OTel[OTEL collector]
    Prom --> Grafana[Grafana]
    Faults[Toxiproxy] --> Runtime
    Faults --> Redis
    Faults --> Store
```

Topology matters because operators do not troubleshoot components one by one in
real incidents. They troubleshoot paths. This page should make it obvious which
links are required, which ones are optional or observability-related, and where
failure isolation can or cannot exist.

## Source of Truth

- `ops/stack/`
- `ops/observe/`
- `ops/k8s/`

## Topology Rules

- the runtime-to-store path is part of the durable serving path
- the runtime-to-Redis path is performance-oriented, not the authoritative data
  path
- Prometheus, Grafana, and OTEL enrich visibility but should not be mistaken
  for serving dependencies
- Toxiproxy is a fault-injection surface and changes topology assumptions only
  during rehearsal or test scenarios
