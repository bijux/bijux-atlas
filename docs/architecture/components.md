# Components

Owner: `architecture`  
Type: `concept`  
Reason to exist: define component responsibilities without implementation drift.

## Responsibility Split

- Ingest components normalize inputs and produce deterministic artifacts.
- Store and registry components resolve immutable release artifacts.
- Query components plan and execute deterministic read paths.
- API and server components expose transport-safe read surfaces.
- Operations components validate deploy, observe, and recover workflows.

## Operational Relevance

Clear component boundaries reduce incident triage time and prevent cross-layer hotfixes.

## Related Pages

- [Architecture](index.md)
- [Boundaries](boundaries.md)
- [Dataflow](dataflow.md)
