---
title: Dataset Workflows
audience: user
type: guide
status: canonical
owner: atlas-docs
last_reviewed: 2026-03-15
---

# Dataset Workflows

Dataset workflows are the bridge between built artifact state and store-backed serving state.

## Dataset Workflow Map

```mermaid
flowchart TD
    Build[Ingest build root] --> Validate[dataset validate]
    Validate --> Verify[dataset verify]
    Verify --> Publish[dataset publish]
    Publish --> Pack[dataset pack]
    Pack --> VerifyPack[dataset verify-pack]
```

## The Important Distinction

```mermaid
flowchart LR
    BuildRoot[Build root] --> ValidateOps[validate and verify]
    BuildRoot --> PublishOps[publish into store]
    Store[Serving store] --> CatalogOps[catalog workflows]
```

Atlas uses dataset commands both before and after publication:

- before publication, they validate or verify build output
- after publication, they help package or inspect durable dataset state

## Most Common Dataset Commands

- `dataset validate`
- `dataset verify`
- `dataset publish`
- `dataset pack`
- `dataset verify-pack`

## Example Workflow

Validate and deeply verify a build root:

```bash
cargo run -p bijux-atlas --bin bijux-atlas -- dataset validate \
  --root artifacts/getting-started/tiny-build \
  --release 110 \
  --species homo_sapiens \
  --assembly GRCh38

cargo run -p bijux-atlas --bin bijux-atlas -- dataset verify \
  --root artifacts/getting-started/tiny-build \
  --release 110 \
  --species homo_sapiens \
  --assembly GRCh38 \
  --deep
```

Publish into a store:

```bash
cargo run -p bijux-atlas --bin bijux-atlas -- dataset publish \
  --source-root artifacts/getting-started/tiny-build \
  --store-root artifacts/getting-started/tiny-store \
  --release 110 \
  --species homo_sapiens \
  --assembly GRCh38
```

## When to Use Pack Operations

Use `dataset pack` and `dataset verify-pack` when you need a portable dataset bundle for transport, validation, or release handling outside the immediate build directory.

## Workflow Advice

- do not skip validation before publication
- treat build roots and serving stores as different lifecycle stages
- use pack verification when moving dataset bundles across trust boundaries

## Purpose

This page explains the Atlas material for dataset workflows and points readers to the canonical checked-in workflow or boundary for this topic.

## Stability

This page is part of the canonical Atlas docs spine. Keep it aligned with the current repository behavior and adjacent contract pages.
