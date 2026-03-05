---
title: Docs Changelog Policy
audience: user
type: reference
stability: stable
owner: docs-governance
last_reviewed: 2026-03-05
tags:
  - docs
  - governance
  - changelog
related:
  - docs/governance/docs-minimization-policy.md
  - docs/governance/docs-surface-boundaries.md
---

# Docs Changelog Policy

Major docs moves must be recorded in changelog entries.

## Required change records

- page merge or split decisions
- canonical path changes
- redirect additions or removals
- navigation section moves
- generated reference model changes

## Entry format

Each entry must include:

- change date
- old path(s)
- new canonical path
- reason
- related redirect entry (if path moved)

## Enforcement

- path moves must include redirect updates in `docs/redirects.json`
- docs reviewers reject major moves without changelog entries
