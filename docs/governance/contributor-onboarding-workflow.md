---
title: Contributor Onboarding Workflow
audience: contributor
type: runbook
stability: stable
owner: bijux-atlas-governance
last_reviewed: 2026-03-05
tags:
  - governance
  - contributors
---

# Contributor onboarding workflow

## Entry criteria

- Signed contributor agreement where required
- Access to repository and CI logs
- Local development environment provisioned

## Onboarding sequence

1. Read governance charter and ownership model.
2. Set up local toolchain and run baseline checks.
3. Complete one documentation-only change.
4. Complete one low-risk code change with tests.
5. Obtain maintainer signoff for independent changes.

## Required baseline commands

```bash
cargo test -p bijux-dev-atlas --no-fail-fast
bijux-dev-atlas governance check --format json
bijux-dev-atlas governance validate --format json
```
