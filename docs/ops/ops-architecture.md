# Ops Architecture

- Owner: bijux-atlas-operations
- Stability: stable

## Diagram

```text
contracts + schemas
        |
        v
ops/inventory ----> ops/{stack,k8s,observe,load,datasets,e2e,report}
        |                              |
        |                              v
        +----------------------> bijux dev atlas ops
                                       |
                                       v
                         artifacts/<run-id>/reports + logs
                                       |
                                       v
                             curated examples in ops/_generated.example
```

## Boundaries

- `ops/`: contracts, inventories, schemas, runbooks, curated evidence.
- `crates/bijux-dev-atlas*`: command behavior, orchestration, validation logic.
- `artifacts/`: runtime outputs and generated execution evidence.

## Determinism

- Input contracts are versioned and reviewed.
- Generated outputs are path-stable and schema-validated.
- Release evidence is assembled from deterministic report sources.
