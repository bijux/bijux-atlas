# Dataset Identity Contract

- Owner: `bijux-atlas-operations`
- Stability: `stable`

## DatasetId

`DatasetId` is the strict triplet:

- `release`
- `species`
- `assembly`

Allowed charset and canonicalization:

- `release`: numeric string only (`[0-9]{1,16}`)
- `species`: snake_case (`[a-z0-9_]{1,64}`), no leading/trailing `_`, no `__`
- `assembly`: `[A-Za-z0-9._]{1,64}`
- Canonical string: `release/species/assembly`

## DatasetKey

Canonical key format used in config/query locks:

`release=<release>&species=<species>&assembly=<assembly>`

Roundtrip is required between `DatasetId` and `DatasetKey`.

## No Implicit Defaults

Ingest and server flows must not rely on implicit default dataset values. Dataset identity must be explicit.

## Artifact Layout

Canonical immutable layout:

`release=<release>/species=<species>/assembly=<assembly>/`

with `inputs/` and `derived/` subtrees.

## Lint

```bash
make dataset-id-lint
```
