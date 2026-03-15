# Ops

`ops/` is the repository-owned source of truth for Atlas operational inputs, inventories, schemas, and generated examples.

- Intent: keep `ops/` as operational data, schemas, inventories, fixtures, and generated evidence examples.
- Machine validation entrypoint: `bijux-dev-atlas ops validate --format json`.
- Focused execution entrypoints: `bijux-dev-atlas ops profiles ...`, `bijux-dev-atlas ops render ...`, `bijux-dev-atlas ops install ...`, and `bijux-dev-atlas ops stack ...`.
- Human walkthroughs and procedures live in `docs/operations/`.

## Design Rules

- Path should tell you whether a file is authored truth, schema, fixture, or generated example.
- Inventories under `ops/inventory/` describe operational authorities; they do not replace validation output.
- Generated examples under `ops/_generated.example/` are illustrative evidence mirrors, not authored truth.
- Runtime effect commands require explicit opt-in flags; static inventory and schema checks do not.
