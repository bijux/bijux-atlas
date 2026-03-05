---
title: Adding Automation Through Dev Atlas
audience: contributor
type: guide
stability: stable
owner: bijux-atlas-governance
last_reviewed: 2026-03-05
tags:
  - automation
  - developer-experience
---

# Adding automation through dev atlas

1. Add a new `bijux-dev-atlas` command under the appropriate domain.
2. Add contract tests for command behavior and surface visibility.
3. Add or update docs to reference the new command.
4. Add a thin Make wrapper only if needed.

Do not add new standalone shell or python automation scripts at repository root.
