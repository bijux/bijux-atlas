# Tutorial Generated Content Inventory

## Tutorials with embedded command blocks

- `docs/tutorials/quickstart.md`
- `docs/tutorials/ingest-dataset.md`
- `docs/tutorials/end-to-end-tutorial.md`
- `docs/tutorials/reproducibility-and-integrity.md`
- `docs/tutorials/docs-build-integration.md`

## Content that should be generated from control-plane sources

- command surface lists (`docs/_generated/command-lists.md`)
- runtime schema snippets (`docs/_generated/schema-snippets.md`)
- OpenAPI snippet lists (`docs/_generated/openapi-snippets.md`)
- operations values/install snippets (`docs/_generated/ops-snippets.md`)
- generated example index (`docs/_generated/examples.md`)

## Generation commands

- `bijux-dev-atlas docs generate command-lists --allow-subprocess --allow-write`
- `bijux-dev-atlas docs generate schema-snippets --allow-write`
- `bijux-dev-atlas docs generate openapi-snippets --allow-write`
- `bijux-dev-atlas docs generate ops-snippets --allow-write`
- `bijux-dev-atlas docs generate examples --allow-write`
