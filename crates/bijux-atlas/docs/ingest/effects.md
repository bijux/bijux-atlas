# Effects Policy

Allowed effects:
- Read local input files (GFF3/FASTA/FAI).
- Write local derived artifacts (SQLite/manifest/anomaly/QC).

Forbidden effects:
- Network access.
- External DB access.
- Process spawning for parsing/transforms.

Ingest must remain a pure file-to-file transform with deterministic outputs.
