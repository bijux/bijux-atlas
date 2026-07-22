---
title: Admin Endpoint Exceptions
audience: operators
type: reference
status: canonical
owner: atlas-docs
last_reviewed: 2026-07-22
---

# Admin Endpoint Exceptions

Atlas keeps recovery, diagnostics, failure-injection, and chaos controls out of
the public route set unless administrative endpoints are explicitly enabled.
The Helm default is `server.adminEndpoints.enabled: false`, which renders
`ATLAS_ENABLE_ADMIN_ENDPOINTS=false`; the runtime default is also false.

When enabled, the server adds all 26 routes below:

| Class | Routes |
| --- | --- |
| dataset and service diagnostics | `/debug/datasets`, `/debug/dataset-health`, `/debug/registry-health`, `/debug/diagnostics`, `/debug/runtime-stats`, `/debug/system-info`, `/debug/build-metadata` |
| configuration and query diagnostics | `/debug/runtime-config`, `/debug/dataset-registry`, `/debug/shard-map`, `/debug/query-planner-stats`, `/debug/cache-stats` |
| cluster control | `/debug/cluster/nodes`, `/debug/cluster-status`, `/debug/cluster/register`, `/debug/cluster/heartbeat`, `/debug/cluster/mode` |
| replica control | `/debug/cluster/replicas`, `/debug/cluster/replicas/health`, `/debug/cluster/replicas/failover`, `/debug/cluster/replicas/diagnostics` |
| recovery and fault control | `/debug/recovery/run`, `/debug/recovery/diagnostics`, `/debug/failure-injection`, `/debug/chaos/run` |
| echo | `/v1/_debug/echo` |

The feature flag changes route registration; it is not, by itself, an
authentication or network-isolation control. An operator must evaluate auth,
service exposure, ingress, and network policy together.

## Current Authorization Gap

The runtime admin classifier currently recognizes 18 of the 26 registered
routes. It omits the four replica routes, both recovery routes,
`/debug/failure-injection`, and `/debug/chaos/run`. Omitted routes are assigned
the ordinary `dataset.read` action and dataset resource kind rather than
`ops.admin` and namespace.

Recognized admin routes are assigned the embedded `operator` principal after
the configured authentication checks. That mapping does not establish that an
external identity provider asserted an operator role. Consequently, enabling
the route group is not evidence of complete operator-only authorization.

Until registration and authorization classification agree, keep the feature
disabled for security-qualified profiles. Any exceptional use needs isolated
reachability and explicit tests for all 26 routes, including unauthorized
negative cases for the eight omitted classifier entries.

## Exception Lifecycle

```mermaid
stateDiagram-v2
    [*] --> Disabled
    Disabled --> Proposed: bounded operational need
    Proposed --> Rejected: safer path exists
    Proposed --> Active: review, registry, and controls pass
    Active --> Revoked: risk or misuse detected
    Active --> Expired: expiry reached
    Active --> Removed: workflow completed
    Revoked --> Disabled: routes and exposure removed
    Expired --> Disabled: routes and exposure removed
    Removed --> Disabled: registry and values reconciled
```

An exception is active only while the registry, selected profile, rendered
configuration, reachability controls, and runtime route set agree. Review
approval without that agreement is not an active exception.

## Default Policy

`ops/k8s/admin-endpoints-exceptions.json` currently contains an empty
`exceptions` array. No profile has a recorded exception. Enabling the endpoints
without an approved registry entry therefore conflicts with the checked-in
operations policy even if the chart renders successfully.

```mermaid
flowchart LR
    Need[Operational need] --> Internal{Keep cluster-internal?}
    Internal -->|yes| Controls[Auth and network controls]
    Internal -->|no| Reject[Reject public exposure]
    Controls --> Record[Register profile, owner, expiry]
    Record --> Render[Render selected profile]
    Render --> Verify[Verify routes and isolation]
    Verify --> Monitor[Retain audit and expiry evidence]
```

## Registry Contract

The current schema accepts only three fields per exception:

| Field | Meaning |
| --- | --- |
| `profile` | exact profile receiving the exception |
| `owner` | accountable owner for removal or renewal |
| `expiresOn` | calendar expiry in `YYYY-MM-DD` form |

The schema does **not** serialize the route, reason, compensating controls, or
evidence references. Those details must accompany the change review, but they
are not recoverable from the registry alone. Do not claim that the current JSON
record provides a complete exception rationale.

## Approval Standard

Approve an exception only when all of these conditions hold:

- the blocked workflow and required route are named
- the route cannot remain disabled for that profile
- authentication and network reachability are explicit
- every registered route is inventoried against its runtime action and resource
  classification
- audit or telemetry coverage can detect use and drift
- the registry names a responsible owner and future expiry
- rendered manifests and a runtime check prove the bounded exposure

Reject an exception when an internal service, temporary port-forward, or
narrower network policy satisfies the workflow without persistent exposure.

## Expiry and Evidence

Before `expiresOn`, the owner must remove the entry and disable the routes or
renew the exception with fresh review evidence. A useful evidence set includes:

- the registry and schema validation result
- the selected values profile and rendered ConfigMap value
- Service, Ingress, and NetworkPolicy resources that bound reachability
- an authenticated positive check and an unauthorized negative check
- a route-parity result covering all routes added by the feature flag
- audit or telemetry output tied to the exercised route

An expired entry, an enabled route without an entry, or an entry for one
profile inherited by another is a failed policy state.

Expiry is a hard boundary. Operators should disable the routes first when an
exception cannot be renewed before its date. Extending only the date without
fresh reachability, authorization, audit, and use-case evidence converts an
exception ledger into permanent exposure and is not a renewal.

## Authorities

- `ops/k8s/admin-endpoints-exceptions.json`
- `ops/schema/k8s/admin-endpoints-exceptions.schema.json`
- `ops/k8s/charts/bijux-atlas/values.yaml`
- `ops/k8s/charts/bijux-atlas/templates/configmap.yaml`
- `ops/k8s/profile-security-contract.json`
- `crates/bijux-atlas-server/src/adapters/inbound/http/router.rs`
