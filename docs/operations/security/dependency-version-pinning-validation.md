---
title: Dependency Version Pinning Validation
audience: user
type: reference
stability: stable
owner: bijux-atlas-security
last_reviewed: 2026-03-05
---

# Dependency Version Pinning Validation

Pinning validation covers:

- lockfile presence and consistency
- GitHub Actions SHA pinning
- container digest pinning for governed images

Validation command:

```bash
bijux-dev-atlas security validate --format json
```
