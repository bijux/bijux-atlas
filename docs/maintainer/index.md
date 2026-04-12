---
title: Maintainer Home
audience: maintainers
type: index
status: canonical
owner: atlas-docs
last_reviewed: 2026-04-12
---

# Maintainer

The maintainer handbook is the control-plane handbook for `bijux-atlas-dev`.

It will hold the deep documentation for `bijux-dev-atlas`, `makes/`, docs
governance, GitHub workflow ownership, release support, and repository checks.

```mermaid
flowchart LR
    Maintainer["Maintainer"]
    Maintainer --> Workspace["Workspace and tooling"]
    Maintainer --> Automation["Automation surfaces"]
    Maintainer --> Governance["Policy and governance"]
    Maintainer --> Delivery["Release and CI"]
    Maintainer --> Docs["Docs and workflow ownership"]
```

## Scope

Use this handbook when the question is about how the repository is operated and
maintained as a governed system rather than how the Atlas product behaves at
runtime.

## What Comes Next

The maintainer handbook is being rebuilt around `maintainer/bijux-atlas-dev/`
with five durable subdirectories so maintainer-only depth has a clear home and
stops competing with product and operations material.

## Current Paths

The active maintainer slices are:

- `maintainer/bijux-atlas-dev/workspace/`
- `maintainer/bijux-atlas-dev/automation/`
- `maintainer/bijux-atlas-dev/governance/`
- `maintainer/bijux-atlas-dev/delivery/`
- `maintainer/bijux-atlas-dev/workflow-ownership/`
*** Add File: /Users/bijan/bijux/bijux-atlas/docs/maintainer/bijux-atlas-dev/workspace/package-surface.md
---
title: Package Surface
audience: maintainers
type: concept
status: canonical
owner: atlas-docs
last_reviewed: 2026-04-12
---

# Package Surface

`bijux-dev-atlas` is the Rust control-plane package that owns repository
automation, docs governance, reports, and enforcement.
