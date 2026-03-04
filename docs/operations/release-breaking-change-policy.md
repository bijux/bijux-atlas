---
title: Release breaking change policy
audience: operators
type: policy
stability: stable
owner: bijux-atlas-operations
last_reviewed: 2026-03-04
tags:
  - release
  - semver
  - compatibility
related:
  - docs/reference/breaking-change-checklist.md
  - docs/operations/release-compatibility-promises.md
---

# Release breaking change policy

- Breaking API changes require a semver major version.
- Breaking schema changes require a semver major version unless explicitly aliased and migration-safe.
- Compatibility lint policy blocks incompatible patch and minor transitions.
- Upgrade and rollback evidence must be attached before release approval.
- Compatibility table updates are required when release versions change.
