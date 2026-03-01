---
title: Make wrapper surface
audience: contributor
type: reference
stability: stable
owner: platform
last_reviewed: 2026-03-01
tags:
  - make
  - control-plane
related:
  - docs/development/tooling/index.md
  - docs/reference/make.md
---

# Make wrapper surface

## Purpose

`make` is a thin wrapper over `bijux dev atlas`. Keep orchestration logic inside control-plane commands, not inside make recipes.

## Laws

- Public targets come from `make/root.mk:CURATED_TARGETS`.
- Public targets must dispatch to `bijux dev atlas`.
- Make recipes must not become an orchestration engine. Avoid shell pipelines and loops except explicit allowlisted compatibility targets.
- Artifacts must be written under `artifacts/<area>/<run_id>/...`.

## How to add or change a target

1. Add or update a control-plane command in `bijux dev atlas`.
2. Add the wrapper target in make modules.
3. Update `CURATED_TARGETS`.
4. Regenerate `make/target-list.json` with `make make-target-list`.
5. Run `make make-contract-check`.

## Discovery commands

- `bijux dev atlas make list --format json`
- `bijux dev atlas make explain <target> --format json`
- `make help`
