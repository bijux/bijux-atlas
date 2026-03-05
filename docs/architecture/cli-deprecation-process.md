---
title: CLI Deprecation Process
audience: contributor
type: process
stability: stable
owner: bijux-atlas-governance
last_reviewed: 2026-03-05
tags:
  - cli
  - deprecation
---

# CLI deprecation process

1. Propose change in architecture docs and contract files.
2. Add migration entry in [cli-command-migrations.md](./cli-command-migrations.md).
3. Update command surface contracts and help snapshots.
4. Update user-facing and developer-facing CLI docs.
5. Run contract suites and verify no undocumented drift remains.

Commands cannot move between `bijux-atlas` and `bijux-dev-atlas` without a migration record.
