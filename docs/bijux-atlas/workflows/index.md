---
title: Workflows
audience: mixed
type: index
status: canonical
owner: atlas-docs
last_reviewed: 2026-07-22
---

# Workflows

Atlas workflows preserve the boundary between building a dataset and serving a
published release. Each path ends with an observable checkpoint so completion
does not depend on an operator interpreting silence as success.

```mermaid
flowchart LR
    Install[Verify binaries] --> Ingest[Validate and ingest]
    Ingest --> Artifact[Inspect artifact identity]
    Artifact --> Publish[Publish store and catalog state]
    Publish --> Serve[Start runtime]
    Serve --> Query[Run structured queries]
    Query --> Observe[Retain result and diagnostics]
```

## Choose a Path

| Outcome | Workflow | Completion signal |
| --- | --- | --- |
| verify an installation | [Install and Verify](install-and-verify.md) | expected command identity and version output |
| exercise the product locally | [Run Atlas Locally](run-atlas-locally.md) | runtime starts against explicit local state |
| learn with governed sample data | [Load a Sample Dataset](load-a-sample-dataset.md) | artifact and dataset identities are inspectable |
| build release-shaped data | [Ingest Workflows](ingest-workflows.md) | validation succeeds and a complete manifest exists |
| inspect or select datasets | [Dataset Workflows](dataset-workflows.md) | requested dataset resolves unambiguously |
| publish discoverable identity | [Catalog Workflows](catalog-workflows.md) | catalog and serving store agree on the release |
| start HTTP serving | [Start the Server](start-the-server.md) | health and readiness report the intended state |
| query published content | [Query Workflows](query-workflows.md) | structured output identifies the resolved release |
| verify the first user path | [Run Your First Queries](run-your-first-queries.md) | representative lookup and sequence results succeed |
| diagnose an early failure | [Troubleshoot Early Problems](troubleshoot-early-problems.md) | failed boundary and corrective action are identified |

## Checkpoints That Matter

A workflow result should preserve enough identity to answer:

- which binary and configuration were used;
- which source inputs and validation policy were admitted;
- which artifact manifest and hashes were produced;
- which catalog entry and store location were selected;
- which release a query resolved; and
- which output or diagnostic established the result.

Success at one boundary does not imply success at the next. In particular, an
ingest directory is not a serving store, an artifact manifest is not a catalog
publication record, and process health is not query correctness.

## Exact Surfaces and Production Operations

Flags, environment variables, endpoints, output shapes, and error behavior are
defined under [Interfaces](../interfaces/index.md). Architecture and lifecycle
ownership are under [Runtime](../runtime/index.md). Deployment, security,
observability, load, rollback, and retained operational evidence are governed
by the [Operations handbook](../../bijux-atlas-ops/index.md).
