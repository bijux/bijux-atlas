# Artifact Contract at Store Boundary

Store crate enforces and consumes:
- `manifest.json`
- `gene_summary.sqlite`
- `manifest.lock`

Contract references:
- `docs/artifact-manifest-contract.md`
- `docs/artifact-directory-contract.md`
- `docs/immutability-guarantee.md`

Rules:
- Publish requires checksum verification before finalization.
- Manifest lock must match manifest/sqlite content on read.
- Catalog must be strictly sorted and valid.
