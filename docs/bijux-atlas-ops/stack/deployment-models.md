---
title: Deployment Models
audience: operators
type: guide
status: canonical
owner: atlas-docs
last_reviewed: 2026-07-22
---

# Deployment Models

Atlas deployment models are defined by profile intent, concrete values,
required tools and services, allowed effects, cluster shape, and evidence. They
are not maturity labels. Every supported model preserves the same publication
boundary: the runtime serves an explicit catalog and immutable store state.

## Profile Selection

```mermaid
flowchart TD
    Need[Operational intent] --> Profile[Select named profile]
    Profile --> Inputs[Resolve cluster config, values, tools, services, effects]
    Inputs --> Render[Render Helm resources]
    Render --> Validate[Schema, policy, and conformance checks]
    Validate --> Install[Install, upgrade, or rollback scenario]
    Install --> Evidence[Probe, telemetry, load, and rollout evidence]
```

`ops/stack/profiles.json` defines local stack footprint. `profile-intent.json`
defines why a profile exists and which effects it permits.
`profile-registry.json` adds safety level, required tools, namespaces, services,
and source paths. `ops/k8s/install-matrix.json` binds Kubernetes values to
install, upgrade, rollback, and validation suites.

## Bind the Profile to a Real Target

Profile selection ends when the effective target is identified, not when a
profile name is parsed. Two clusters using the same values can differ in
admission mutation, storage, scheduling, identity, network enforcement, and
external dependency behavior.

| Target property | Why it belongs in qualification |
| --- | --- |
| cluster identity and Kubernetes version | prevents evidence from moving between unrelated or incompatible control planes |
| node classes and failure domains | bounds scheduling, capacity, churn, and availability claims |
| storage classes and object-store service | establishes persistence, consistency, latency, backup, and recovery ownership |
| ingress, DNS, and network enforcement | identifies the actual client path, TLS boundary, timeouts, and isolation behavior |
| workload identity and secret providers | establishes principals, credential generations, rotation, and revocation paths |
| telemetry destinations and retention | establishes whether required evidence is queryable for the decision window |
| admission and policy controllers | exposes defaults or mutations between rendered intent and admitted objects |

```mermaid
flowchart LR
    Profile[Named Atlas profile] --> Render[Rendered intent]
    Target[Target capability record] --> Admit[Admission and deployment]
    Render --> Admit
    Admit --> Observe[Observed resources and service]
    Observe --> Qualify{Target-specific evidence passes?}
    Qualify -->|yes| Accepted[Bound deployment claim]
    Qualify -->|no| Hold[Hold or narrow the claim]
```

The target capability record must be versioned with the evidence it qualifies.
An environment label such as `prod` or `kind` is not a substitute for these
observations. Requalify after a cluster upgrade, storage migration, ingress or
identity change, admission-policy change, or failure-domain redesign.

## Local and Validation Profiles

| Profile | Intended use | Required dependencies | Evidence limit |
| --- | --- | --- | --- |
| `minimal` | smallest supported contract footprint | Kind cluster and rendered manifests | not production or resilience proof |
| `small` | quick constrained local validation | Kind cluster and rendered manifests | no full telemetry claim |
| `ci` | deterministic automated validation | Kind cluster and rendered manifests | restricted effects; targeted install evidence |
| `kind` | baseline local cluster and smoke checks | cluster, rendered manifests, health endpoint | standard local integration evidence |
| `dev` | iterative local development | cluster, rendered manifests, health endpoint | allows networked local workflows |
| `developer` | workstation cluster with standard ergonomics | cluster, rendered manifests, health endpoint | development evidence only |
| `perf` | performance baseline and autoscaling checks | cluster, rendered manifests, metrics server, health endpoint | strict profile; requires load and metrics evidence |

The concrete stack manifest currently expands `ci` and `local` with the Atlas
chart, operations namespace, MinIO, and Redis. The `kind` stack adds Prometheus,
Grafana, and OpenTelemetry. A profile appearing in an intent registry does not
mean every operational component is present; the generated dependency graph is
the resolved component evidence.

## Kubernetes Delivery Profiles

The install matrix also covers `ingress`, `multi-registry`, `offline`, and
`prod` values. These are Kubernetes delivery concerns rather than extra local
stack classes.

| Delivery concern | Required proof |
| --- | --- |
| baseline install | schema-valid values, rendered resources, install suite |
| ingress | explicit ingress values, routing and security review, nightly evidence |
| multi-registry | pinned image sources and pull behavior across registries |
| offline | locally available images and artifacts, no hidden network dependency |
| performance | metrics prerequisites, autoscaling configuration, governed load evidence |
| production | security context, network policy, resource, availability, backup, and rollback review |

## Promotion Is Not Inheritance

```mermaid
flowchart LR
    Local[Local or CI evidence] --> Candidate[Release candidate]
    Candidate --> Rendered[Target-profile render]
    Rendered --> Security[Security and policy validation]
    Security --> Runtime[Target-environment probes and telemetry]
    Runtime --> Load[Relevant load and failure scenarios]
    Load --> Rollback[Upgrade and rollback evidence]
    Rollback --> Promote[Promotion decision]
```

Evidence from a smaller profile proves only that profile's contract. It cannot
be inherited as production readiness. Promotion requires the target profile's
rendered identity and all evidence demanded by its security, telemetry,
capacity, and recovery posture.

## Qualification Boundary by Model

The same runtime can move through several deployment models while the
environmental claim changes. Use each model for the conclusions it can
actually establish.

| Model | What is held authoritative | Suitable conclusions | Conclusions that require another model |
| --- | --- | --- | --- |
| repository and render validation | source, schema, profiles, chart inputs, and rendered objects | deterministic configuration, policy conformance, and dependency declaration | runtime behavior, storage behavior, or recovery |
| `ci` composition | pinned local composition and restricted automated effects | repeatable install and targeted integration behavior | full observability, target capacity, and external-service ownership |
| `kind` composition | local cluster, expanded observability stack, MinIO, Redis, and selected profile | Kubernetes lifecycle, local dependency integration, probes, telemetry, and bounded load experiments | target durability, identity, network, and failure-domain behavior |
| target environment | observed target resources plus owned and external dependency contracts | target security, capacity, availability, rollout, rollback, and recovery claims exercised there | behavior in a different target or an unexercised failure domain |

```mermaid
flowchart LR
    R["Repository and render evidence"] --> C["CI composition evidence"]
    C --> K["Kind lifecycle and integration evidence"]
    K --> T["Target-environment evidence"]
    R -. unchanged inputs may transfer .-> T
    C -. environment conclusions must be repeated .-> T
    K -. local dependency conclusions must be repeated .-> T
```

Transfer a result only when its authority and relevant inputs remain unchanged.
For example, schema validity may transfer with identical bytes, while storage
durability, workload identity, ingress behavior, and capacity must be
established against the services and failure domains that own those properties.

## Evidence Transfer

| Earlier evidence | Reusable conclusion | Must be repeated for the target |
| --- | --- | --- |
| source and schema validation | authored inputs satisfy repository contracts | target-specific values and overlays |
| deterministic render for another profile | chart templates can produce valid objects | target resource inventory and policy result |
| local functional result | product path works for recorded local dependencies | target networking, storage, identity, and traffic path |
| lower-scale load result | scenario and query pack can execute | target capacity, autoscaling, saturation, and recovery budgets |
| prior release rollback | recovery workflow has a known shape | candidate-to-previous compatibility in the target profile |

Evidence is reusable when the authority and unchanged inputs are identical.
Environment-dependent conclusions are not portable merely because the runtime
binary is the same.

## External Dependency Handoff

The Atlas profile controls only the assets it owns. A production decision must
also name the operator for every service supplied by the environment:

| Dependency | Atlas needs | Environment owner must establish |
| --- | --- | --- |
| object store | immutable object reads and catalog access | durability, credentials, encryption, backup, restore, and consistency behavior |
| ingress and DNS | stable routing to eligible instances | TLS policy, name ownership, timeout behavior, and traffic rollback |
| workload identity | least-privilege access to dependencies | principal lifecycle, credential rotation, revocation, and audit trail |
| telemetry backend | accepted logs, metrics, and traces | retention, query availability, access control, and loss detection |
| cluster services | scheduling, storage, metrics, and network enforcement | supported versions, capacity, admission policy, and failure ownership |

Record the provider, service identity, escalation route, recovery objective,
and evidence source for each dependency. “Managed elsewhere” is a deployment
fact, not an ownership answer. If an external service is required for
readiness, correctness, or promotion evidence, its failure policy belongs in
the target profile's operating record.

## Non-Negotiable Boundaries

- Serve from published store state, never directly from an ingest build root.
- Keep runtime, dataset, chart, values, dependency, and toolchain identity
  reviewable together.
- Treat readiness as traffic admission, not as complete performance proof.
- Record relaxed security, network, or admin behavior as an explicit exception.
- Prove rollback against the same release and profile identities used for
  promotion.

The selected model should also document who owns external dependencies. A
managed object store, ingress, identity provider, or telemetry backend may sit
outside the Atlas composition while remaining inside the release decision.

Continue with [Install Matrix](../kubernetes/install-matrix.md),
[Render and Validate](../kubernetes/render-and-validate.md), and
[Rollout Safety](../kubernetes/rollout-safety.md).
