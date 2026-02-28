# Dataflow

Owner: `architecture`  
Type: `concept`  
Reason to exist: define the ingest to serve dataflow and ownership boundaries.

## Flow

`ingest -> artifact publish -> registry/store -> query -> API serve`

## Stages

1. Ingest validates and normalizes source inputs.
2. Artifacts are published with immutable identity and checksums.
3. Store and registry expose release-indexed datasets.
4. Query resolves explicit dataset identity and executes deterministic plans.
5. API serves read-only responses with stable contract envelopes.

## Operational Relevance

Each stage has independent verification and rollback behavior, which is required for incident response.
