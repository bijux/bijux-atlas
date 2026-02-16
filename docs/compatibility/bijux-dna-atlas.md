# Compatibility Statement: bijux-dna -> bijux-atlas

`bijux-dna` can emit datasets consumable by `bijux-atlas` when artifact contracts are respected.

## Required Contract Elements

Producer output from `bijux-dna` must include:

- Canonical dataset identity: `{release, species, assembly}`.
- Manifest with strict schema (unknown fields rejected).
- Input checksums (GFF3/FASTA/FAI) and derived checksum (SQLite).
- SQLite schema version compatible with atlas query expectations.

## Compatibility Rules

- Dataset IDs must pass atlas normalization rules.
- Manifest schema version must be declared and validated.
- Published artifacts are immutable once marked published.
- `latest` aliases are explicit and not implicit in API queries.

## Future Adapter

A dedicated adapter can map `bijux-dna` internal outputs to atlas SSOT paths:

- Normalize paths into atlas artifact directory contract.
- Enforce strict checksum reconciliation.
- Emit deterministic catalog entries sorted canonically.

This adapter is planned but not required when `bijux-dna` emits atlas-native contracts directly.
