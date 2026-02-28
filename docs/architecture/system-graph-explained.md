# System graph explained

- Owner: `architecture`
- Type: `concept`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@7f82f1b0`
- Reason to exist: explain the canonical runtime graph and how to read it.

## Diagram

![Atlas runtime graph from server through query and store to immutable artifacts](../_assets/diagrams/system-graph.svg)

## How to read this graph

- Left to right follows request flow from serve to query and store.
- Dashed dependency surfaces represent contract constraints, not runtime data writes.
- Artifact/storage nodes are immutable read sources during serving.

## Text explanation

The graph shows a strict read path for serving: API requests resolve through query and store layers to immutable artifacts. Runtime writes occur only in ingest/publish workflows outside serve path.

## Terminology used here

- Artifact: [Glossary](../glossary.md)
- Query: [Glossary](../glossary.md)

## Next steps

- [Architecture map](architecture-map.md)
- [Dataflow](dataflow.md)
- [Runtime data model](runtime-data-model.md)
