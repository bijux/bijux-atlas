---
title: Stack
audience: operators
type: index
status: canonical
owner: atlas-docs
last_reviewed: 2026-07-22
---

# Atlas Stack

The Atlas stack is the declared environment around the serving runtime: cluster
substrate, chart, object store, cache, observability services, profiles, and
pinned container inputs. Each layer has a different authority. Keeping those
authorities separate prevents a development convenience from becoming an
accidental production dependency.

## Stack Authority

```mermaid
flowchart LR
    I["Pinned images and tools"] --> C["Component configuration"]
    C --> D["Service dependency contract"]
    P["Profile registry and intent"] --> G["Generated composition graph"]
    D --> G
    G --> H["Health surfaces"]
    H --> E["Stack evidence"]
    E --> O{"Operate, hold, or recover"}
```

The profile registry declares seven operating intentions: `ci`, `dev`,
`developer`, `kind`, `minimal`, `perf`, and `small`. The generated composition
graph is narrower and currently covers `ci`, `kind`, and `local`. `local` is a
composition identity in `stack.toml`; it is not an entry in the seven-profile
policy registry.

## Supported Composition Graphs

| Composition | Cluster shape | Required components | Optional components included |
| --- | --- | --- | --- |
| `ci` | small Kind cluster | chart, observability namespace, MinIO, Redis | none |
| `local` | small Kind cluster | chart, observability namespace, MinIO, Redis | none |
| `kind` | normal Kind cluster | chart, observability namespace, MinIO, Redis | Prometheus, Grafana, OpenTelemetry collector |

The service dependency contract marks the chart, observability namespace,
MinIO, and Redis as critical for all three generated compositions. The
observability services are noncritical and required only by the `kind`
composition. Required here means required by the checked-in stack contract; it
does not imply that every external Atlas deployment must use the same backend.

## Plane Ownership

| Plane | Components | Authority |
| --- | --- | --- |
| substrate | Kind or external cluster, namespaces, storage classes, network | supplies execution and isolation primitives |
| delivery | Helm chart, profiles, image pins, and generated composition | declares what should run |
| serving | Atlas runtime, catalog, immutable store, and cache | resolves and serves dataset state |
| observation | Prometheus, Grafana, collector, probes, and diagnostics | measures behavior and preserves investigation context |
| recovery | retained artifacts, configuration, backups, rollback targets | restores a previously verified operating identity |

The stack contract names a composition; it does not transfer authority between
planes. Prometheus cannot establish dataset integrity. Redis cannot become a
catalog. A Helm release record cannot prove the runtime answered correctly.
Keep these boundaries visible. Diagnose one plane at a time. Then test the
cross-plane effect before restoring traffic or promotion authority.

## Failure Propagation

```mermaid
flowchart LR
    Client[Client] --> Service[Atlas service]
    Service --> Store[Object store]
    Service --> Cache[Cache]
    Service --> Telemetry[Telemetry pipeline]
    Store -->|artifact unavailable| NotReady[Readiness or request failure]
    Cache -->|unavailable| Degraded[Bypass or bounded degradation]
    Telemetry -->|unavailable| Blind[Reduced diagnostic confidence]
    NotReady --> Decision[Hold, drain, or recover]
    Degraded --> Decision
    Blind --> Decision
```

Dependency criticality and failure behavior are separate properties. A
critical store failure can remove serving eligibility. Cache failure may be
recoverable within explicit latency and load budgets. Telemetry failure may
leave user traffic intact while making promotion unsafe because required
signals cannot be observed.

## Dependency Failure Budget

Before accepting a dependency, define its availability role and bounded
degradation:

| Property | Operator decision |
| --- | --- |
| startup requirement | may a new instance become ready without it? |
| steady-state requirement | may existing traffic continue, and for how long? |
| correctness impact | can failure change dataset or response meaning? |
| capacity impact | what backend pressure, queueing, or latency follows? |
| evidence impact | which required signals or audit records disappear? |
| recovery path | retry, bypass, cached-only mode, fail closed, or restore |

Exercise the selected degradation under load. A fallback that works for one
request may collapse under concurrency or hide stale state during an outage.

| Dependency condition | Runtime concern | Operator concern |
| --- | --- | --- |
| store unavailable or inconsistent | artifact resolution and query correctness | stop promotion; verify readiness, manifest identity, and recovery |
| cache unavailable or saturated | latency, backend pressure, and overload | confirm bypass behavior and capacity before continuing |
| telemetry pipeline unavailable | loss or delay of required signals | preserve local diagnostics and treat evidence gaps explicitly |
| version or pin drift | unreviewed bytes in the composition | reconcile declared and observed identity before release use |

## Route by Operating Question

| Question | Read |
| --- | --- |
| Which components are actually governed? | [Stack Components](stack-components.md) |
| What fails when a dependency is unavailable? | [Dependency Graph](dependency-graph.md) |
| How does a request cross runtime and dependencies? | [Service Topology](service-topology.md) |
| Which deployment shape fits the environment? | [Deployment Models](deployment-models.md) |
| What does each local profile permit and require? | [Local Stack Profiles](local-stack-profiles.md) |
| Which Kind substrate is selected? | [Kind Clusters](kind-clusters.md) |
| How do environment values alter the base? | [Environment Overlays](environment-overlays.md) |
| How are cache and store failures handled? | [Cache and Store Operations](cache-and-store-operations.md) |
| What is needed without network access? | [Offline Assets](offline-assets.md) |
| Which external bytes are pinned? | [Toolchain Pins](toolchain-pins.md) |

## Change Rule

A component, dependency edge, profile, or pin change is an operational contract
change. Update the owning input, regenerate stack inventories and the dependency
graph, review failure isolation and profile impact, then preserve the resulting
evidence. A successful pod start does not prove that the intended composition
was assembled.
