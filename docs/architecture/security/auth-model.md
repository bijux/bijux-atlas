---
title: Security Auth Model
audience: contributor
type: concept
stability: stable
owner: bijux-atlas-security
last_reviewed: 2026-03-03
tags:
  - architecture
  - security
  - auth
related:
  - docs/operations/security/deploy-behind-auth-proxy.md
  - docs/operations/security/index.md
  - docs/reference/contracts/security.md
---

# Security Auth Model

- Owner: `bijux-atlas-security`
- Type: `concept`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Reason to exist: define the canonical authentication and access-boundary stance for Atlas.

## Purpose

State the default trust boundary so deployers, reviewers, and contributors do not infer conflicting
security behavior.

## Default stance

Atlas is **internal by default**.

That means:

- Atlas is intended to serve trusted institutional, cluster-local, or mesh-local callers.
- Direct unauthenticated public exposure is not the default deployment model.
- Built-in request authentication is supported for controlled internal deployments, but perimeter
  policy still matters.

## Supported authentication

Atlas supports built-in request authentication with these methods:

- `api-key`
- `oidc`
- `mtls`

The runtime configuration surface is `auth.mode`, exposed as `ATLAS_AUTH_MODE`.

## Deployment boundary requirement

Even when built-in authentication is enabled, the recommended production boundary is an ingress
auth layer or service-mesh policy in front of Atlas. Built-in auth protects the application
surface; it does not replace institutional edge controls, rate shaping, or centralized identity.

See [Deploy Behind Auth Proxy](../../operations/security/deploy-behind-auth-proxy.md) for the
operational pattern.

## Recommended auth placement

Preferred order:

1. Ingress auth proxy for human and institutional client entrypoints.
2. Service mesh policy for service-to-service paths.
3. Built-in `api-key` where the caller set is small and tightly governed.

For proxy-verified `oidc` and `mtls`, Atlas expects the trusted boundary to forward only approved
identity headers after authentication succeeds.

## Principal vocabulary

Atlas uses these principal categories:

- `user`
- `service-account`
- `operator`
- `ci`

Their durable registry lives in `configs/security/principals.yaml`.

## Action vocabulary

Atlas uses these durable action IDs:

- `catalog.read`
- `dataset.read`
- `dataset.ingest`
- `ops.admin`

Their durable registry lives in `configs/security/actions.yaml`.

## Resource vocabulary

Atlas uses these durable resource scopes:

- `dataset-id`
- `namespace`
- `tenant`
- `project`

Their durable registry lives in `configs/security/resources.yaml`.
