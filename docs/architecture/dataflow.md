# Dataflow

- Owner: `architecture`
- Type: `concept`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@6856280c`
- Reason to exist: define the canonical runtime flow from ingest to serving.

## Runtime Flow

`ingest -> artifact -> registry/store -> query -> serve`

## Read Path vs Write Path

- Write path: ingest validates source inputs and publishes immutable artifacts.
- Read path: query and API serve immutable release data with deterministic semantics.

## Determinism Guardrails

- Artifact generation is canonicalized.
- Registry merges are deterministic.
- Query and API contracts pin response semantics.

## Failure Modes

- Ingest validation failure blocks publication.
- Registry conflict blocks release alias progression.
- Store or query pressure triggers degraded but explicit behavior.

## Operational Relevance

Each stage has independent verification and rollback controls for safer deployments.

## What This Page Is Not

This page is not a runbook and not a schema reference.

## Example

```text
Input file accepted -> artifact published -> release alias updated -> query served.
```

## What to Read Next

- [Architecture](index.md)
- [Boundaries](boundaries.md)
- [Storage](storage.md)
- [Glossary](../glossary.md)

## Document Taxonomy

- Audience: `contributor`
- Type: `concept`
- Stability: `stable`
- Owner: `architecture`
