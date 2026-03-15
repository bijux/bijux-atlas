---
title: Make Wrapper Migration Notes
audience: contributor
type: reference
stability: stable
owner: platform
last_reviewed: 2026-03-05
tags:
  - make
  - migration
---

# Make wrapper migration notes

## Canonical delegation targets

- `suites-all` -> `bijux-dev-atlas suites run --suite deep ...` and `bijux-dev-atlas suites run --suite contracts ...`
- `ops-fast` -> `bijux-dev-atlas suites run --suite ci_fast ...`
- `ops-pr` -> `bijux-dev-atlas suites run --suite ci_pr ...`
- `ops-nightly` -> `bijux-dev-atlas suites run --suite ci_nightly ...`
- `tests-all` -> `bijux-dev-atlas tests run --mode all ...`
- `docs-build` -> `bijux-dev-atlas docs build ...`
- `docs-serve` -> `bijux-dev-atlas docs serve ...`
- `ops-validate` -> `bijux-dev-atlas ops validate ...`
- `release-plan` -> `bijux-dev-atlas release plan ...`
- `openapi-generate` -> `bijux-dev-atlas api contract ...`

## Deprecated compatibility wrappers

Where temporary aliases remain, they must point to a canonical delegation command and be removed only with a migration entry.
