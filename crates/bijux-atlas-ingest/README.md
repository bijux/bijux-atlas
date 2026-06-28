# bijux-atlas-ingest

`bijux-atlas-ingest` owns the Atlas ingest engine: GFF3 and FASTA decoding,
normalization, anomaly evaluation, SQLite artifact materialization, and ingest
focused benchmarks or fixtures.

Use this crate when you need:

- deterministic Atlas ingest execution
- ingest artifact and anomaly report generation
- ingest normalization replay and diff support
- ingest-owned tests and benchmark surfaces

Public references:

- Project docs: <https://bijux.github.io/bijux-atlas/>
- Rust API docs: <https://docs.rs/bijux-atlas-ingest/latest/bijux_atlas_ingest/>
- Source repository: <https://github.com/bijux/bijux-atlas>
