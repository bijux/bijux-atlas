---
title: Threat Model Scope
audience: contributor
type: concept
stability: stable
owner: bijux-atlas-security
last_reviewed: 2026-03-04
tags:
  - architecture
  - security
  - threat-model
---

# Threat Model Scope

## In Scope

- API request authentication bypass attempts
- authorization escalation and permission confusion
- sensitive data exposure via logs and diagnostics
- credential leakage and secret misuse in runtime paths
- integrity tampering for dataset artifacts and manifests
- transport downgrades and boundary spoofing

## Out Of Scope

- host-level kernel compromise
- physical hardware attacks
- external identity provider internals

## Security Invariants

- protected routes must have authenticated principal context
- authorization checks must run before protected action execution
- sensitive values must never be emitted as plaintext in telemetry artifacts
- audit events must include stable event type and actor identity metadata
