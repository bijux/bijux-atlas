# Dataset Operations Reference

Owner: `bijux-atlas-operations`  
Type: `reference`  
Reason to exist: provide factual command and artifact references for dataset ingest and promotion.

## Command Surface

- `atlas dataset validate`
- `atlas dataset publish`
- `atlas catalog promote`
- `atlas catalog rollback`

## Core Inputs

- artifact root
- catalog file
- dataset identity (`ops/release/species/assembly`)

## Canonical dataset manifest

- The canonical dataset ID registry is `ops/datasets/manifest.json`.
- The governed schema for that file is `ops/schema/datasets/manifest.schema.json`.
- Kubernetes install profiles that set `cache.pinnedDatasets` must use only IDs declared in that manifest.

## Current canonical IDs

- `110/homo_sapiens/GRCh38`
- `111/homo_sapiens/GRCh38`

## Adding a dataset ID

1. Add the new entry to `ops/datasets/manifest.json`.
2. Add any required fixture or provenance references that the manifest entry points to.
3. Re-run `bijux dev atlas contracts ops --mode effect --allow-subprocess --filter-contract OPS-DATASET-001`.
4. If an install profile pins the new dataset, update that profile only after the manifest change is present.

## Example pinned dataset list

```yaml
cache:
  pinnedDatasets:
    - 110/homo_sapiens/GRCh38
```
