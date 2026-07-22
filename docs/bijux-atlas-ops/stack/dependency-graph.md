---
title: Dependency Graph
audience: operators
type: concept
status: canonical
owner: atlas-docs
last_reviewed: 2026-07-22
---

# Dependency Graph

The dependency graph answers two questions: which services must exist for a
declared composition, and which health signal proves each dependency is usable.
It is generated from the service dependency contract and stack composition,
not inferred from every configuration file under `ops/stack/`.

## Declared Graph

```mermaid
flowchart TD
    Profile{"Composition"}
    Profile --> Chart["Atlas Helm chart<br/>critical"]
    Profile --> Namespace["Observability namespace<br/>critical"]
    Profile --> MinIO["MinIO object store<br/>critical"]
    Profile --> Redis["Redis cache<br/>critical"]
    Profile -. kind only .-> Prometheus["Prometheus<br/>noncritical"]
    Profile -. kind only .-> Grafana["Grafana<br/>noncritical"]
    Profile -. kind only .-> OTel["OpenTelemetry collector<br/>noncritical"]
    Chart --> Install["k8s.install-matrix"]
    Namespace --> NamespaceReady["observe.namespace-ready"]
    MinIO --> MinIOReady["stack.minio-ready"]
    Redis --> RedisReady["stack.redis-ready"]
    Prometheus --> PromReady["observe.prometheus-ready"]
    Grafana --> GrafanaReady["observe.grafana-ready"]
    OTel --> OTelReady["observe.otel-ready"]
```

## Criticality Semantics

| Class | Admission rule | Failure interpretation |
| --- | --- | --- |
| Critical | Required health surface must pass before the composition is accepted. | The stack is unavailable or incomplete for that profile. |
| Noncritical | Component may enrich the composition without defining the serving path. | Telemetry or visualization is degraded; application health must be judged separately. |

Noncritical does not mean ignorable. A missing collector or Prometheus instance
can invalidate a rollout, incident, or SLO claim even while queries succeed.
Criticality describes stack viability; the evidence required for a particular
decision may be stricter.

## Composition Criticality Is Not Runtime Authority

The checked-in graph marks Redis critical for the `ci`, `kind`, and `local`
compositions because those compositions promise to assemble and verify the
Redis component. The Atlas serving model still treats Redis response state as
optional and non-authoritative. These statements answer different questions.

| Question | Owning contract | Redis answer |
| --- | --- | --- |
| Was the declared local composition assembled completely? | stack dependency contract | Redis must exist and its health surface must pass. |
| Can Atlas remain correct without response-cache acceleration? | runtime cache policy and degradation evidence | Atlas may bypass Redis within policy and capacity. |
| Does Redis determine dataset release truth? | catalog, manifest, and store contracts | No; cached entries must remain bound to verified authority. |
| Can a release claim full local-stack evidence while Redis is absent? | selected composition and evidence receipt | No; the environment differs from the declared composition. |

Do not weaken the composition report to hide a missing component, and do not
make application readiness depend on cache content merely because the local
composition requires the Redis service. A deployment that intentionally omits
Redis needs a different resolved composition identity and a qualified cold-path
capacity result.

## Dependency Roles

```mermaid
flowchart LR
    Server[Atlas server] --> Serving[Serving dependencies]
    Server --> Acceleration[Acceleration dependencies]
    Server --> Evidence[Evidence dependencies]
    Delivery[Delivery controller] --> Registry[Chart, image, package, and policy sources]
    Serving --> Catalog[Catalog and object store]
    Acceleration --> Local[Dataset and response caches]
    Acceleration --> Redis[Optional Redis response cache]
    Evidence --> Telemetry[Collector, metrics, logs, traces]
```

The declared stack graph is a composition graph, not the whole dependency
universe. A service can be absent from startup ordering yet remain necessary
for release retrieval, credential resolution, backup recovery, or promotion
evidence. Record those external dependencies in the environment's operating
inventory instead of adding false runtime edges to the generated graph.

| Role | Failure policy |
| --- | --- |
| serving authority | fail closed when correct dataset identity or immutable bytes cannot be established |
| acceleration | bypass or shed within policy; never substitute different data |
| operational evidence | continue serving only within the environment's telemetry-degradation policy; hold claims requiring missing evidence |
| delivery and recovery | block new promotion or recovery when immutable artifact or trust identity cannot be resolved |

## Edge Contracts

A node inventory cannot explain who spends the deadline, amplifies traffic, or
decides that a recovered dependency is safe. Record each selected edge from the
caller's point of view.

| Edge | Caller-owned contract | Failure boundary | Recovery proof |
| --- | --- | --- | --- |
| server to catalog | generation, freshness, timeout, and fallback policy | no admissible dataset selection | expected generation resolves through serving credentials |
| server to artifact store | identity, integrity, concurrency, retry, and breaker budget | bytes unavailable or untrusted | named object verifies inside operating budgets |
| server to Redis | lookup, fill, namespace, TTL, cardinality, and bypass limits | shared acceleration degraded | cold path stays correct; return causes no miss storm |
| server to telemetry path | export queue, timeout, loss, retention, and evidence need | decision evidence incomplete | signals remain queryable for the decision window |
| delivery controller to registry | digest, trust, credential scope, and offline source | artifact identity unavailable | immutable artifact resolves and passes trust policy |

The generated service graph proves membership and health-surface selection for
the checked-in composition. The edge contract supplies direction, caller
ownership, and failure semantics. Both are required before a reachable service
can satisfy a deployment claim.

## Dependency States

```mermaid
stateDiagram-v2
    [*] --> Declared
    Declared --> Resolved: version and configuration selected
    Resolved --> Reachable: network and credentials succeed
    Reachable --> Usable: owning health contract passes
    Usable --> Degraded: latency, errors, or freshness breach
    Degraded --> Usable: recovery verified
    Reachable --> Failed: semantic or integrity check fails
    Resolved --> Failed: connection or identity fails
```

Reachability is weaker than usability. A TCP connection does not establish
that the correct bucket, catalog, cache namespace, metrics source, or API
version is available. Health surfaces should test the narrowest semantics
needed by the dependent runtime without performing destructive work.

## Retain a Dependency Receipt

For every selected edge, retain enough identity to distinguish the intended
dependency from a reachable substitute:

| Receipt field | Why it matters |
| --- | --- |
| logical role. | Separates serving authority, acceleration, evidence, and delivery dependencies. |
| endpoint and namespace. | Identifies the actual network and tenancy boundary used by the profile. |
| version or digest. | Prevents a healthy but incompatible service from satisfying the edge. |
| credential identity and scope. | Proves least-privilege access without retaining secret values. |
| semantic health result. | Demonstrates the required bucket, catalog, cache namespace, or telemetry path. |
| timeout and retry owner. | Makes the caller deadline and amplification budget reviewable. |
| degradation and recovery result. | Shows what happened when the edge failed and when it returned. |

Bind the receipt to the profile, release, cluster, and observation window.
Reusing a health result after an endpoint, credential, version, or policy
change creates a new unverified edge even when the dependency name is the same.

## Failure Ownership

| Failure | Primary owner | Cross-domain impact |
| --- | --- | --- |
| selected version or digest cannot resolve | stack and release inputs | installation and reproducibility |
| credentials or network path fail | deployment and security profile | readiness and incident response |
| object store returns missing or inconsistent artifact | store and release identity | correctness, readiness, and recovery |
| cache is unavailable | runtime cache policy | latency, backend load, and overload |
| collector or metrics backend is unavailable | observability pipeline | promotion and diagnosis confidence |

## Profile Membership

`ci` and `local` contain four critical components: chart, namespace, MinIO, and
Redis. `kind` contains the same critical set plus Prometheus, Grafana, and the
OpenTelemetry collector. The generated graph does not define `dev`,
`developer`, `minimal`, `perf`, or `small`; those remain profile-policy entries
until a composition graph explicitly binds their components.

Toxiproxy has configuration and a pinned image for controlled failure work, but
it is absent from the current service dependency contract and generated graph.
Do not present it as a required stack service. Evidence from a Toxiproxy drill
must name the separately assembled component and fault configuration.

## Reviewing a Graph Change

Review every added or removed node for profile membership, criticality, health
surface, startup ordering, credentials, network access, failure isolation, and
offline availability. Then regenerate `ops/stack/generated/dependency-graph.json`
and confirm it agrees with `stack.toml` and the dependency contract.

A missing edge can hide a real dependency. An extra edge can make an optional
service look mandatory. Either mismatch is a release-evidence defect.

## Failure-Budget Review

For every edge, define connection and operation timeouts, retry ownership,
circuit-breaking or shedding behavior, maximum queueing, and the signal that
declares recovery. Nested retries across runtime, proxy, and dependency can
multiply traffic and extend failure beyond the caller's deadline; one layer
must own the retry budget.

Test the graph in both directions. Dependency loss must produce the declared
runtime behavior, and dependency recovery must restore service without stale
identity, retry storms, or an unbounded cache refill. A health check that only
opens a socket cannot prove either property.
