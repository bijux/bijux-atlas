# Effects And Boundaries

Concept ID: `concept.effects-boundary`

- Owner: `bijux-atlas-server`

Effect policy for Phase 1:

- Pure domain and transformation crates: `core`, `model`, `ingest`, `query`.
  - No external process spawning.
  - No direct filesystem/network side effects.
- Store crate (`bijux-atlas-store`): allowed to perform storage I/O.
- Server crate (`bijux-atlas-server`): allowed to perform wiring/bootstrap I/O only.
- API crate (`bijux-atlas-api`): pure read-service layer over query outputs.
  - No raw GFF3/FASTA reading.
  - No process spawning.

```mermaid
flowchart LR
  subgraph Pure
    core[core]
    model[model]
    query[query]
  end
  subgraph Effectful
    store[store io]
    server[server runtime io]
  end
  api[api mapping]

  core --> model
  model --> query
  query --> api
  api --> server
  server --> store
```

Override/escape hatches are forbidden unless explicitly documented and approved in policy docs.
