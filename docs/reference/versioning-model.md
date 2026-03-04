---
title: Versioning model
audience: contributors
type: reference
stability: stable
owner: platform
last_reviewed: 2026-03-05
tags:
  - reference
  - release
related:
  - docs/reference/workspace-versioning-policy.md
  - release/crates-v0.1.toml
---

# Versioning model

- Workspace follows unified semver for the `v0.1` line.
- Publishable crates share one version number per release tag.
- API removals require semver break evaluation via `release semver check`.
- Compatibility and rollout policy are reviewed before release publication.
