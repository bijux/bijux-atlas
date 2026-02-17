# Artifact Manifest Contract

Manifest is strict JSON (`serde(deny_unknown_fields)`) with:

- Dataset identity (`release`, `species`, `assembly`)
- Input checksums: GFF3, FASTA, FAI
- Derived checksum: SQLite
- Basic stats: gene/transcript/contig counts
- Versions: manifest version + DB schema version
- Dataset signature hash: `dataset_signature_sha256` (Merkle-style over table content)
- Schema evolution note: `schema_evolution_note`
- Derived column lineage map: `derived_column_origins`

Unknown fields are rejected.
