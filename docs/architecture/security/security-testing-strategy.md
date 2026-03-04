---
title: Security Testing Strategy
audience: contributor
type: concept
stability: stable
owner: architecture
last_reviewed: 2026-03-04
tags:
  - architecture
  - security
  - testing
related:
  - docs/architecture/security/security-architecture.md
  - docs/operations/security/security-testing-and-monitoring.md
---

# Security Testing Strategy

Atlas uses layered security verification so changes are rejected before release:

1. Contract validation and static policy checks in CI for auth, authorization, data protection, and supply-chain controls.
2. Parser and request-surface fuzzing for query, ingest, authentication, and authorization paths.
3. Adversarial API traffic suites that cover malicious payloads, injection attempts, and overload behavior.
4. Release evidence gates for vulnerability budget, pinned dependencies, and audit artifacts.

## Acceptance baseline

- No panics or 5xx regressions under malformed and adversarial input corpora.
- No unreviewed HIGH/CRITICAL vulnerability exposure beyond documented budget controls.
- No policy bypass across authn/authz decision points.
- Stable audit evidence generated for every release candidate.

## Mapping

- Security policy validation: `bijux-dev-atlas security validate`
- Dependency and vulnerability policy: `bijux-dev-atlas security validate` and `cargo audit`
- Attack simulations: `ops/load/k6/suites/security-*.js`
- Runtime posture checks: `docs/operations/security/runtime-security-monitoring.md`
