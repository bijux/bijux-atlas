---
title: Image tag deprecation policy
audience: operators
type: policy
stability: stable
owner: platform
last_reviewed: 2026-03-04
tags:
  - operations
  - docker
related:
  - docs/operations/image-compatibility-policy.md
---

# Image tag deprecation policy

- Version tags remain immutable and must never be retagged to different digests.
- Deprecated tags are announced in release notes and retained for the declared support window.
- `latest` is optional and non-authoritative; consumers should pin version or digest.
- Deprecated tags should be clearly marked in release documentation before retirement.
