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

## Non-Negotiable Boundaries

- Serve from published store state, never directly from an ingest build root.
- Keep runtime, dataset, chart, values, dependency, and toolchain identity
  reviewable together.
- Treat readiness as traffic admission, not as complete performance proof.
- Record relaxed security, network, or admin behavior as an explicit exception.
- Prove rollback against the same release and profile identities used for
  promotion.

Continue with [Install Matrix](../kubernetes/install-matrix.md),
[Render and Validate](../kubernetes/render-and-validate.md), and
[Rollout Safety](../kubernetes/rollout-safety.md).
