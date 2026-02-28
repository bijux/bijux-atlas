# Runtime data model

- Owner: `architecture`
- Type: `concept`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@ff4b8084`
- Reason to exist: provide one-page overview of artifact, registry, and serving-store runtime data relationships.

## Core entities

- Artifact: immutable output from validated ingest inputs.
- Registry Record: release metadata mapping release IDs and aliases to artifact locations.
- Serving Store: query-time indexed store over immutable release data.

## Relationship model

1. Ingest builds immutable artifact.
2. Registry records artifact identity and release mapping.
3. Serving store reads registry-resolved artifact data for query/API responses.

## Guarantees

- Artifact immutability.
- Deterministic registry merge behavior.
- Read-path semantics independent of transient cache state.

## Limits and non-goals

- Runtime data model does not permit in-place mutation of published artifact state.
- Runtime data model does not allow ambiguous alias mapping outcomes.

## What to Read Next

- [Dataflow](dataflow.md)
- [Storage](storage.md)
- [Reference schemas](../reference/schemas.md)

## Terminology used here

- Artifact: [Glossary](../glossary.md)
- Registry: [Glossary](../glossary.md)

## Document Taxonomy

- Audience: `contributor`
- Type: `concept`
- Stability: `stable`
- Owner: `architecture`
