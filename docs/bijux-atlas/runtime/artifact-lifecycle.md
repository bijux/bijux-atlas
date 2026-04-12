---
title: Artifact Lifecycle
audience: mixed
type: concept
status: canonical
owner: atlas-docs
last_reviewed: 2026-04-12
---

# Artifact Lifecycle

Atlas artifacts move through a predictable lifecycle:

- build from validated inputs
- verify the produced shape
- publish into a serving store
- expose through catalog and runtime lookup
- compare or retire through release-aware workflows

## Why This Matters

The lifecycle is the hinge between ingest work and serving work. If artifact
state is unclear, both runtime behavior and operations drift.

## Lifecycle States

Atlas artifact handling is easiest to reason about as five named states:

1. `source-selected`: the repository knows which governed inputs are in scope
2. `build-complete`: Atlas has emitted the release-shaped artifact set
3. `verification-complete`: checks confirm the artifact shape and metadata
4. `published`: the artifact set exists in the serving store and catalog path
5. `servable`: runtime lookup can resolve the published dataset identity

Readers should treat `build-complete` and `servable` as different milestones.
An artifact can exist locally without being part of the serving truth yet.

## Promotion Rule

The intended Atlas path is not:

- ingest something locally
- point the server directly at incidental build output

The intended path is:

- build a deterministic artifact set
- verify it
- publish it into the serving store
- serve catalog-backed lookup from that published state

That rule is what keeps Atlas artifact-first instead of quietly turning into a
mutable local-workspace server.

## Identity and Traceability

Each lifecycle step should preserve enough identity to answer:

- which release or dataset this artifact belongs to
- which source inputs produced it
- whether the artifact has been verified
- whether the runtime should consider it publishable or already published

If that identity is missing, Atlas loses most of the value of immutable
artifact handling because operators and maintainers can no longer distinguish
candidate output from serving truth.

## Failure Patterns

Common lifecycle mistakes include:

- treating temporary build output as if it were published serving state
- comparing artifacts without preserving the release identity they belong to
- skipping verification and then debugging runtime behavior from ambiguous input
- deleting or moving state in ways that break traceability between build and publish steps

## Related Pages

- [Ingest Architecture](ingest-architecture.md)
- [Serving Store Model](serving-store-model.md)
- [Storage Architecture](storage-architecture.md)
- [Artifact and Store Contracts](../contracts/artifact-and-store-contracts.md)
