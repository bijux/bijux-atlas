---
title: Make Surface Evolution Policy
audience: contributor
type: policy
stability: stable
owner: platform
last_reviewed: 2026-03-05
tags:
  - make
  - governance
---

# Make surface evolution policy

- `make` stays a thin delegation interface; orchestration belongs in `bijux-dev-atlas`.
- Public wrappers must be added to `make/root.mk:CURATED_TARGETS` only when they are stable.
- Wrapper changes must update `make/target-list.json` and `configs/make/public-targets.json`.
- Wrapper changes must pass `bijux-dev-atlas make wrappers verify` and make-domain contracts.
- Deprecated wrappers require explicit migration notes and replacement commands.
