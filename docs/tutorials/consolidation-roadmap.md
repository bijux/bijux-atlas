---
title: Tutorial Consolidation Roadmap
audience: user
type: concept
stability: stable
owner: docs-governance
last_reviewed: 2026-03-05
tags:
  - tutorial
  - consolidation
related:
  - docs/tutorials/index.md
  - docs/tutorials/style-guide.md
---

# Tutorial Consolidation Roadmap

Target structure keeps a short learning path and moves repeated details into references.

Merge candidates:

- `debug-pipeline.md` + `debugging-with-logs.md` + `debugging-with-traces.md` into one debugging guide.
- `reading-metrics.md` + `interpreting-dashboards.md` into one observability tutorial.
- `inspect-artifacts.md` + `analyzing-evidence-bundles.md` into one evidence tutorial.

Reference sinks:

- command and wrapper surfaces in `docs/_generated/command-lists.md`
- API route excerpts in `docs/_generated/openapi-snippets.md`
- schema excerpts in `docs/_generated/schema-snippets.md`
- Helm values inventory in `docs/_generated/ops-snippets.md`
