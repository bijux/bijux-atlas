---
title: Trust Boundaries
audience: contributor
type: concept
stability: stable
owner: bijux-atlas-security
last_reviewed: 2026-03-04
tags:
  - architecture
  - security
---

# Trust Boundaries

## Boundary Definitions

1. external caller boundary: request enters Atlas from an untrusted network path
2. ingress identity boundary: proxy or gateway performs identity assertion before forwarding
3. service runtime boundary: authenticated request is handled by Atlas runtime components
4. storage boundary: data and artifacts are read from storage backends
5. operator boundary: privileged operational commands and diagnostics access

## Boundary Rules

- identity headers are trusted only from approved ingress boundaries
- internal handlers trust principal context only after authentication middleware
- diagnostics and audit outputs crossing operator boundaries must be redacted
