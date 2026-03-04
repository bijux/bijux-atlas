---
title: Ops evidence bundles
owner: platform
stability: stable
last_reviewed: 2026-03-05
---

# Ops evidence bundles

Use `bijux dev atlas ops evidence collect` to produce release evidence and per-run ops evidence artifacts.

Required evidence artifacts:
- install evidence
- render evidence
- validate evidence
- install matrix report
- render matrix report
- schema coverage report
- network policy coverage report
- RBAC coverage report
- inventory snapshot
- tool versions

Verification:
- `bijux dev atlas ops evidence verify`
- Verifies file presence, checksums, redaction patterns, and tarball membership.
