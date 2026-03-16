---
title: User Guide Troubleshooting
audience: user
type: troubleshooting
status: canonical
owner: atlas-docs
last_reviewed: 2026-03-15
---

# Troubleshooting

This page is for problems that occur after the initial getting-started path: command confusion,
workflow mismatches, rejected queries, or uncertainty about which Atlas layer you are interacting
with.

## Troubleshooting Map

```mermaid
flowchart TD
    Problem[Problem] --> Config[Configuration and output issue]
    Problem --> Dataset[Dataset or catalog issue]
    Problem --> Query[Query issue]
    Problem --> Runtime[Server runtime issue]
    Problem --> Policy[Policy issue]
```

This troubleshooting map is meant to reduce scope quickly. User-guide problems usually become much
easier once you identify whether the issue belongs to configuration, dataset state, runtime state,
query shape, or policy.

## The Fastest Diagnostic Question

Ask: which lifecycle stage am I in?

```mermaid
flowchart LR
    Inputs[Source inputs] --> Build[Build root]
    Build --> Store[Serving store]
    Store --> Runtime[Running server]
    Runtime --> Client[Client request]
```

This lifecycle diagram is the fastest mental reset when things feel blurry. Many “Atlas is broken”
reports are actually one stage being mistaken for another.

Most confusion comes from mixing those stages:

- build-root issues are not the same as serving-store issues
- serving-store issues are not the same as runtime issues
- runtime issues are not the same as query-shape issues

## Common User Errors

- trying to start the server against an ingest build root
- expecting broad queries to succeed without explicit selectors
- treating a missing catalog as a generic server failure
- using human-readable output in automation and then depending on its exact wording

## Practical Debug Order

1. confirm the command surface with `--help`
2. confirm the relevant build root or serving store exists
3. confirm dataset identity values are correct
4. confirm runtime health with `/healthz`, `/readyz`, and `/v1/version`
5. validate query shape before assuming the data is missing

## A Reliable Troubleshooting Rule

Do not fix three layers at once. Pick the first failing layer in the sequence above, make one
change, and rerun the same check before you move on.

## When to Leave This Section

Move to:

- [Operations](../04-operations/index.md) for deployment or production incidents
- [Reference](../07-reference/index.md) for exact endpoint or config lookup
- [Architecture](../05-architecture/index.md) if the problem is really conceptual confusion

## Purpose

This page explains the Atlas material for troubleshooting and points readers to the canonical checked-in workflow or boundary for this topic.

## Stability

This page is part of the canonical Atlas docs spine. Keep it aligned with the current repository behavior and adjacent contract pages.
