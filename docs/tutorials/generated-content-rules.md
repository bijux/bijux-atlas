---
title: Tutorial Generated Content Rules
audience: user
type: reference
stability: stable
owner: docs-governance
last_reviewed: 2026-03-05
tags:
  - tutorial
  - docs
  - generated-content
related:
  - docs/tutorials/generated-content-workflow.md
  - docs/tutorials/style-guide.md
---

# Tutorial Generated Content Rules

Generated snippet rules:

- Files in `docs/_generated/` are written only by `bijux-dev-atlas docs generate ...`.
- Every generated markdown file must start with the generator header marker.
- Generated files must be referenced by at least one docs page.
- Tutorials must include generated snippets instead of pasting long command output.
- Manual edits to generated files are not allowed; regenerate and commit instead.
