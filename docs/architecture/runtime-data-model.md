# Runtime Data Model

- Owner: `architecture`
- Type: `concept`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@ff4b8084`
- Reason to exist: provide one-page overview of artifact, registry, and serving-store runtime data relationships.

## Core Entities

- Artifact: immutable output from validated ingest inputs.
- Registry Record: release metadata mapping release IDs and aliases to artifact locations.
- Serving Store: query-time indexed store over immutable release data.

## Relationship Model

1. Ingest builds immutable artifact.
2. Registry records artifact identity and release mapping.
3. Serving store reads registry-resolved artifact data for query/API responses.

## Guarantees

- Artifact immutability.
- Deterministic registry merge behavior.
- Read-path semantics independent of transient cache state.

## What to Read Next

- [Dataflow](dataflow.md)
- [Storage](storage.md)
- [Reference Schemas](../reference/schemas.md)

## Document Taxonomy

- Audience: `contributor`
- Type: `concept`
- Stability: `stable`
- Owner: `architecture`
