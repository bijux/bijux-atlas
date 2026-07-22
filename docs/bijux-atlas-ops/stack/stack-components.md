---
title: Stack Components
audience: operators
type: reference
status: canonical
owner: atlas-docs
last_reviewed: 2026-07-22
---

# Stack Components

Atlas stack components fall into three categories: serving dependencies,
observability dependencies, and test-only fault infrastructure. The category
determines whether absence blocks stack admission and what evidence must be
preserved.

## Component Catalog

| Component | Role | Contract status | Health authority |
| --- | --- | --- | --- |
| Atlas Helm chart | Renders and deploys the runtime service | Critical in `ci`, `kind`, and `local` | Kubernetes install matrix |
| Observability namespace | Owns monitoring resources and isolation | Critical in `ci`, `kind`, and `local` | `observe.namespace-ready` |
| MinIO | Object-store dependency for governed stack compositions | Critical in `ci`, `kind`, and `local` | `stack.minio-ready` |
| Redis | Cache dependency for governed stack compositions | Critical in `ci`, `kind`, and `local` | `stack.redis-ready` |
| Prometheus | Metric collection and rule evaluation | Noncritical, included by `kind` | `observe.prometheus-ready` |
| Grafana | Operator visualization | Noncritical, included by `kind` | `observe.grafana-ready` |
| OpenTelemetry collector | Trace and telemetry intake | Noncritical, included by `kind` | `observe.otel-ready` |
| Toxiproxy | Controlled dependency-failure injection | Not in the stack dependency contract | Scenario-specific evidence |

The Kind node image is a substrate pin rather than a service node. Kubernetes
resource templates are owned by the chart, while MinIO, Redis, Prometheus,
Grafana, OpenTelemetry, and Toxiproxy configuration live under their respective
`ops/stack/` directories.

## Configuration Is Not Membership

The presence of a YAML file does not place a component in a profile. Membership
comes from `ops/stack/stack.toml` and the generated dependency graph.
Criticality and health surfaces come from
`ops/stack/service-dependency-contract.json`. Image identity comes from the
generated version manifest sourced from the central pin inventory.

```mermaid
flowchart LR
    Y["Component YAML"] --> A{"Composition includes it?"}
    A -->|no| U["Available but unassembled"]
    A -->|yes| C{"Dependency contract entry?"}
    C -->|yes| H["Apply criticality and health surface"]
    C -->|no| S["Scenario-specific component"]
```

## Component Acceptance

For every assembled component, retain its immutable image reference,
configuration digest, profile membership, namespace, health result, and
relevant credentials or network-policy identity. For a noncritical component,
also state which operational claims become unavailable when it fails.

Do not replace a failing dependency with a mock and retain the same evidence
claim. A mock, local filesystem, external Redis, or different object-store
backend defines a different composition and must be identified as such.

## State and Recovery Ownership

Component criticality does not determine whether its state is durable. Recovery
must follow the data owner:

| Surface | State class | Failure response | Recovery proof |
| --- | --- | --- | --- |
| Atlas service | replaceable process with release-bound configuration | drain or replace the affected revision | intended image, configuration, dataset, and traffic eligibility restored |
| immutable object store | release and dataset authority when selected by the profile | stop publication and protect existing objects | object identity, manifest, access policy, and post-recovery verification |
| Redis | disposable acceleration unless a profile declares otherwise | degrade explicitly or bypass under policy | cache identity, bounded miss behavior, and refill without release mutation |
| Prometheus and collector | operational evidence path | hold promotion when required signals are blind | scrape or intake continuity plus a post-recovery control event |
| Grafana | diagnostic presentation | use raw metrics and preserve dashboard outage | datasource, dashboard revision, and diagnostic path restored |
| Toxiproxy | test-only fault mechanism | abort the scenario and remove injected conditions | target topology restored and fault no longer active |

Do not restore disposable cache bytes as release truth or treat retained
telemetry as a source for rebuilding dataset state. Every recovery action must
name which plane it changes: serving, durable data, acceleration, evidence, or
test control.

## Dependency Degradation Decision

```mermaid
flowchart TD
    Failure[Component failure confirmed] --> Identity[Bind profile, release, component, and time]
    Identity --> Critical{Critical for selected claim?}
    Critical -->|yes| Remove[Remove traffic or hold mutation]
    Critical -->|no| Degrade[Enter declared degraded mode]
    Remove --> Recover[Recover owning state]
    Degrade --> Observe[Prove remaining claims and blind spots]
    Recover --> Verify[Verify identity and behavior]
    Observe --> Verify
```

A noncritical label permits a bounded loss of capability; it does not permit
silent disappearance. Record the unavailable dashboards, traces, comparison,
or fault evidence and narrow the operating claim accordingly.

Before returning a component to service, verify its configured identity,
dependency edges, credentials, network reachability, health surface, and any
state it owns. A process-level health response cannot prove that an object
store contains the intended immutable release or that telemetry continuity was
preserved.

See [Dependency Graph](dependency-graph.md) for failure boundaries and
[Toolchain Pins](toolchain-pins.md) for immutable external identities.
