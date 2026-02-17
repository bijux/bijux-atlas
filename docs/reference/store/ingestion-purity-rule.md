# Ingestion Purity Rule

Ingestion is a pure file transformation.

Allowed:

- Read local input files (GFF3, FASTA, FAI).
- Write local output files (SQLite, manifest, copied inputs under artifact root).

Forbidden:

- Network calls.
- External DB access.
- External process spawning.
