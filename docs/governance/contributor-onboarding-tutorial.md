---
title: Contributor Onboarding Tutorial
audience: contributor
type: tutorial
stability: stable
owner: bijux-atlas-governance
last_reviewed: 2026-03-05
tags:
  - governance
  - contributor
---

# Contributor onboarding tutorial

## Objective

Ship a first safe change with full governance checks.

## Steps

1. Run baseline checks:
```bash
cargo test -p bijux-dev-atlas --no-fail-fast
bijux-dev-atlas governance check --format json
```
2. Make a docs-only change and open a pull request.
3. Address review feedback and confirm governance checks stay green.
4. Make a low-risk code change with test coverage.
5. Request maintainer signoff and merge.
