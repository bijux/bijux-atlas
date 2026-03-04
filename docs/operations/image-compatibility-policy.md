---
title: Image compatibility policy
audience: operators
type: policy
stability: stable
owner: platform
last_reviewed: 2026-03-04
tags:
  - operations
  - docker
related:
  - docs/reference/crates.md
  - docs/reference/docker.md
---

# Image compatibility policy

- Runtime image tag `vX.Y.Z` is aligned with workspace release version `X.Y.Z`.
- Runtime image SHA tag points to an immutable build from the same release source commit.
- Image-level compatibility follows crate/API compatibility guarantees for the same release version.
- Breaking crate/API changes require a new semver-major image tag.
