---
title: Admin Endpoints Exceptions
audience: operators
type: reference
status: canonical
owner: atlas-docs
last_reviewed: 2026-04-13
---

# Admin Endpoints Exceptions

Exceptions around admin-facing endpoints are tracked as schema-backed policy so
security review and operations review can reason about the same surface. The
default state is no exception: `ops/k8s/admin-endpoints-exceptions.json`
currently records an empty `exceptions` array, which means every administrative
surface is expected to stay disabled, isolated, or authenticated by default.

## Purpose

Use this page when a Kubernetes profile needs a narrowly scoped deviation from
the normal Atlas rule that administrative endpoints stay off the public path and
out of routine user traffic.

## Source of Truth

- `ops/k8s/admin-endpoints-exceptions.json`
- `ops/schema/k8s/admin-endpoints-exceptions.schema.json`
- `ops/k8s/profile-security-contract.json`
- `ops/k8s/examples/networkpolicy/`

## What Is Governed

An admin endpoint exception is a governed object, not an ad hoc note in a pull
request. The governing schema defines the object shape and keeps the exception
set reviewable as operational policy.

An exception record should identify:

- the endpoint, route, or capability being exposed
- the profile or environment where the exception applies
- the reason the default security posture is insufficient
- the compensating controls that keep the exception bounded
- the expiry date and revalidation trigger
- the evidence attached to justify the exception

## Decision Criteria

Only approve an exception when all of these are true:

- the operator can name the blocked workflow that requires the endpoint
- the endpoint cannot stay disabled or cluster-internal
- the selected profile security contract still holds for auth, network policy,
  and auditability
- the exposure is time-bounded and the owner is explicit
- there is validation evidence showing the exception behaves as intended

Reject the request when a normal chart value, service profile, or internal-only
network policy would solve the problem without widening the attack surface.

## Review Workflow

1. The proposer adds or updates the exception object in
   `ops/k8s/admin-endpoints-exceptions.json`.
2. The proposer links the profile, endpoint, reason, expiry, and compensating
   controls in the change review.
3. Security and operations review the request against
   `ops/schema/k8s/admin-endpoints-exceptions.schema.json` and the relevant
   profile security contract.
4. The approver confirms the change is reflected in network policy or service
   wiring and that observability coverage exists for misuse or drift.
5. Before expiry, the owner must either remove the exception or revalidate it
   with fresh evidence.

## How to Validate

- Validate the exception file against
  `ops/schema/k8s/admin-endpoints-exceptions.schema.json`.
- Confirm the owning profile still satisfies
  `ops/k8s/profile-security-contract.json`.
- Check the matching network policy examples in `ops/k8s/examples/networkpolicy/`
  to verify the exception is still narrower than the normal cluster-aware
  egress or ingress model.
- Confirm logs, alerts, or dashboard coverage exists if the endpoint changes
  security posture.

## Failure Modes

- an endpoint remains open after the exception should have expired
- a profile inherits the exception without being named explicitly
- compensating controls are documented but not rendered into manifests
- drift removes the alerting or audit coverage that justified the exception
- a rollout keeps the exception while the underlying operational need no longer
  exists

## Evidence Produced

Attach evidence that explains why the exception exists and proves it is bounded:

- schema validation output for the exception record
- rendered manifest or network policy evidence showing the scope of exposure
- security review notes tied to the owning profile
- observability evidence showing the endpoint is monitored and auditable

## Related Contracts and Assets

- `ops/k8s/admin-endpoints-exceptions.json`
- `ops/schema/k8s/admin-endpoints-exceptions.schema.json`
- `ops/k8s/profile-security-contract.json`
- `ops/k8s/examples/networkpolicy/cluster-aware.yaml`
- `ops/k8s/examples/networkpolicy/custom.yaml`
