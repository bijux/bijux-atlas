# Artifact Contract at Store Boundary

Store crate enforces and consumes:
- `manifest.json`
- `gene_summary.sqlite`
- `release_gene_index.json`
- `manifest.lock`
- Optional sharded outputs (backward compatible with monolithic mode):
- `catalog_shards.json`
- `gene_summary.<shard>.sqlite`

Contract references:
- `docs/artifact-manifest-contract.md`
- `docs/artifact-directory-contract.md`
- `docs/product/immutability-and-aliases.md`

Rules:
- Publish requires checksum verification before finalization.
- Manifest lock must match manifest/sqlite content on read.
- Catalog must be strictly sorted and valid.
- Sharding modes are internal implementation details: `per-seqid` or bounded `N` partitions.
- API/query contract remains stable regardless of physical shard layout.
