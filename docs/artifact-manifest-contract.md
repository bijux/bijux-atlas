# Artifact Manifest Contract

Manifest is strict JSON (`serde(deny_unknown_fields)`) with:

- Dataset identity (`release`, `species`, `assembly`)
- Input checksums: GFF3, FASTA, FAI
- Derived checksum: SQLite
- Basic stats: gene/transcript/contig counts
- Versions: manifest version + DB schema version

Unknown fields are rejected.
