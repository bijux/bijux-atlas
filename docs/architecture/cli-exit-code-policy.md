---
title: CLI Exit Code Policy
audience: contributor
type: policy
stability: stable
owner: bijux-atlas-governance
last_reviewed: 2026-03-05
tags:
  - cli
  - exit-codes
---

# CLI exit code policy

## User CLI (`bijux-atlas`)

- Exit codes are part of public compatibility guarantees.
- Usage errors return the stable usage code.
- Structured machine-readable errors must remain stable.

## Developer CLI (`bijux-dev-atlas`)

- Exit codes are stable for CI automation and contracts.
- Contract/check commands return non-zero on policy violations.
- Developer command failures must include deterministic machine-readable output in JSON mode.

## Shared policy

- Command docs must describe expected failure classes.
- Breaking exit code behavior requires a migration note in [cli-command-migrations.md](./cli-command-migrations.md).
