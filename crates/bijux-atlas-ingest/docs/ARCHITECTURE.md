# Architecture

## Architecture

Modules:
- `gff3`: streaming parse and strict attribute decoding.
- `fai`: contig length loading and validation.
- `extract`: gene/transcript extraction and anomaly classification.
- `sqlite`: deterministic DB materialization and indexes.
- `manifest`: manifest/QC report construction.
