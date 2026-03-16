---
title: Getting Started
audience: mixed
type: index
status: canonical
owner: atlas-docs
last_reviewed: 2026-03-15
---

# Getting Started

This section gets Atlas running with real commands and real repository fixtures.

The goal is not to teach every feature. The goal is to give you a successful first run that makes the rest of the documentation meaningful.

## The First-Run Path

```mermaid
flowchart TD
    A[Install and verify] --> B[Run Atlas locally]
    B --> C[Load a sample dataset]
    C --> D[Publish and promote into a serving store]
    D --> E[Start the server]
    E --> F[Run first queries]
    F --> G[Troubleshoot early problems if needed]
```

## What You Will Have at the End

- a working Atlas CLI invocation
- a sample build root under `artifacts/`
- a sample serving store under `artifacts/`
- a running local Atlas server
- successful queries against the local runtime

```mermaid
flowchart LR
    Fixtures[Repo fixtures] --> Build[Local ingest build root]
    Build --> Publish[Publish and catalog promote]
    Publish --> Store[Serving store under artifacts/]
    Store --> Server[Local server]
    Server --> Queries[First queries]
```

## Pages in This Section

- [Install and Verify](install-and-verify.md)
- [Run Atlas Locally](run-atlas-locally.md)
- [Load a Sample Dataset](load-a-sample-dataset.md)
- [Start the Server](start-the-server.md)
- [Run Your First Queries](run-your-first-queries.md)
- [Troubleshoot Early Problems](troubleshoot-early-problems.md)

```mermaid
flowchart LR
    GS[Getting Started] --> Install[Install]
    GS --> Local[Run locally]
    GS --> Sample[Load sample]
    GS --> Server[Start server]
    GS --> Query[First queries]
    GS --> Trouble[Troubleshoot]
```

## Ground Rules

- commands prefer repository-relative paths so you can follow them from the workspace root
- sample data comes from committed test fixtures rather than invented fake commands
- output roots go under `artifacts/`, not inside crate directories

## Purpose

This page explains the Atlas material for getting started and points readers to the canonical checked-in workflow or boundary for this topic.

## Stability

This page is part of the canonical Atlas docs spine. Keep it aligned with the current repository behavior and adjacent contract pages.
