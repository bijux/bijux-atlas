---
title: How To Add A Make Wrapper
audience: contributor
type: guide
stability: stable
owner: platform
last_reviewed: 2026-03-05
tags:
  - make
  - contributor-guide
---

# How to add a make wrapper

1. Add or update the `bijux-dev-atlas` command first.
2. Add a single-line target in `make/*.mk` delegating to that command.
3. Add target name to `make/root.mk:CURATED_TARGETS` if public.
4. Regenerate surface artifacts with `make make-target-list`.
5. Run `bijux-dev-atlas make wrappers verify` and `make make-contract-check`.
