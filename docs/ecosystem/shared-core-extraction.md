# Shared Core Extraction

Reusable canonical utilities are extracted to dedicated repository:

- repo: `/Users/bijan/bijux/bijux-core`
- crate: `bijux-core`
- modules:
  - `canonical` (stable JSON hashing/sorting)
  - `cursor` (signed opaque cursor encoding/decoding)

`bijux-atlas` keeps local compatibility surfaces during migration, while cross-project reuse should target `bijux-core` for new integrations.
