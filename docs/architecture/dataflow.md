# Dataflow

Owner: `architecture`  
Type: `concept`  
Reason to exist: define canonical flow from ingest to API serving.

## Flow

`ingest -> publish -> registry/store -> query -> api`

## Operational Relevance

Each stage has independent verification and rollback controls for safer deployments.

## Related Pages

- [Architecture](index.md)
- [Components](components.md)
- [Storage](storage.md)
