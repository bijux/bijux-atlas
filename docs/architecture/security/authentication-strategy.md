---
title: Authentication Strategy
audience: contributor
type: concept
stability: stable
owner: architecture
last_reviewed: 2026-03-04
tags:
  - architecture
  - security
  - authentication
related:
  - docs/architecture/security/security-architecture.md
  - docs/architecture/security/identity-access-data-models.md
---

# Authentication Strategy

## Purpose

Define how Atlas proves request identity before authorization decisions are evaluated.

## Supported Methods

Atlas supports these production authentication methods:

1. `api-key`: static or rotated key material for service-to-service access.
2. `token`: signed bearer token with issuer, audience, scope, and expiry validation.
3. `oidc`: identity forwarded by a trusted boundary proxy.
4. `mtls`: service identity forwarded from certificate-authenticated boundary.
5. `disabled`: only for local development or controlled diagnostics.

## Method Selection Rules

1. Production deployments should run one primary method for data-plane routes.
2. Health and liveness routes remain exempt to preserve operability.
3. Administrative routes require operator identity and must never rely on anonymous access.
4. `disabled` mode is non-production and must be explicitly configured.

## API Key Model

1. Raw keys are generated once and never stored in plain form.
2. Server-side validation compares hashes, not raw key bytes.
3. Keys carry ownership metadata and expiration metadata.
4. Rotation keeps overlap between old and new keys during rollout.
5. Revoked keys are denied immediately.

## Token Model

1. Token validation pipeline enforces structure, signature, expiry, and policy claims.
2. Issuer and audience validation are explicit allow-list checks.
3. Scope extraction provides action-level authorization inputs.
4. Token IDs (`jti`) support immediate revocation.
5. Expired or not-yet-valid tokens are rejected with authentication errors.

## Optional OAuth Integration

Atlas supports OAuth/OIDC integration through a trusted boundary proxy.
The proxy is responsible for interactive login and token exchange.
Atlas consumes verified identity headers and applies route policy locally.

## Authentication Context Contract

Every authenticated request produces an immutable context:

1. principal classification
2. subject identifier
3. issuer
4. scopes
5. authentication mechanism
6. request identity metadata (request id, client ip, user agent)

This context is the handoff boundary to authorization and audit logging.
