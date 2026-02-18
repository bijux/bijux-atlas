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

`ops/store/release=<release>/species=<species>/assembly=<assembly>/`

with `inputs/` and `derived/` subtrees.

Required manifest/db identity fields:

- `artifact_version` (artifact schema stream, separate from release number)
- `manifest.schema_version` and `manifest.db_schema_version` (required and equal)
- dataset identity in manifest and SQLite `atlas_meta`:
  - `dataset_release`
  - `dataset_species`
  - `dataset_assembly`
- `manifest.input_hashes`: `gff3`, `fasta`, `fai`, `policy`
- `manifest.toolchain_hash` (rust toolchain + lockfile digest)
- `manifest.created_at` is allowed metadata and excluded from determinism hash

## Lint

```bash
make dataset-id-lint
```
