---
title: Run Atlas Locally
audience: mixed
type: how-to
status: canonical
owner: atlas-docs
last_reviewed: 2026-03-15
---

# Run Atlas Locally

Running Atlas locally is easiest when you separate the workflow into three areas:

- source fixtures
- built artifacts
- runtime processes

## Local Layout

```mermaid
flowchart LR
    Fixtures[crates/bijux-atlas/tests/fixtures] --> Build[Ingest and validation]
    Build --> BuildRoot[artifacts/getting-started/tiny-build]
    BuildRoot --> Publish[dataset publish and catalog promote]
    Publish --> Store[artifacts/getting-started/tiny-store]
    Store --> Server[bijux-atlas-server]
    Server --> Client[curl or browser]
```

## Prepare a Local Workspace

```bash
mkdir -p artifacts/getting-started/tiny-build
mkdir -p artifacts/getting-started/tiny-store
mkdir -p artifacts/getting-started/server-cache
```

Keep all throwaway local outputs under `artifacts/`. Do not create crate-local scratch directories.

## Inspect the Main Surfaces

```bash
cargo run -p bijux-atlas --bin bijux-atlas -- --help
cargo run -p bijux-atlas --bin bijux-atlas-server -- --help
```

## Understand the Local Loop

```mermaid
flowchart TD
    A[Build sample dataset] --> B[Validate dataset root]
    B --> C[Publish into serving store]
    C --> D[Promote into catalog]
    D --> E[Start server with serving store]
    E --> F[Query local endpoints]
    F --> G[Inspect logs and metrics]
```

The local development loop is not “start the server and hope.” It is:

1. build a sample dataset into an artifact root
2. validate the resulting build root
3. publish and promote into a serving store
4. point the server at that serving store
5. query the resulting release state

## Why Atlas Prefers This Loop

Atlas is artifact-centric. That means local runtime behavior should be tested against built dataset state, not against half-prepared source inputs or improvised mutable state.

```mermaid
flowchart LR
    Bad[Build root directly treated as serving state] --> Drift[Confusion and drift]
    Good[Build root published into serving store] --> Confidence[Deterministic local behavior]
```

## Recommended Local Sequence

- follow [Load a Sample Dataset](load-a-sample-dataset.md)
- then follow [Start the Server](start-the-server.md)
- then follow [Run Your First Queries](run-your-first-queries.md)

## Local Success Criteria

You are running Atlas locally in the intended way when:

- ingest outputs live under `artifacts/`
- dataset validation succeeds on the build root
- the serving store contains published artifacts plus catalog state
- the server points at the serving store rather than raw fixtures or the ingest build root
- queries return data for the release you just built
