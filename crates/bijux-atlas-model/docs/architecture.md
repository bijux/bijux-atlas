# Architecture

## Architecture

`bijux-atlas-model` defines domain and contract types used by ingest/store/query/api layers.

Module responsibilities:
- `dataset`: dataset identifiers and strict canonicalization.
- `gene`: gene identifiers and gene-level policies.
- `manifest`: artifact manifest/catalog contracts.
- `policy`: cross-domain strictness and identifier selection policy.
