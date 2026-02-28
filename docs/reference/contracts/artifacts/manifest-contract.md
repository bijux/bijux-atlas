# Artifact Manifest Contract

- Owner: `docs-governance`

Manifest is strict JSON (`serde(deny_unknown_fields)`) with:

- Dataset identity (`release`, `species`, `assembly`)
- `artifact_version` (artifact schema stream, not release number)
- `schema_version` + `db_schema_version` (both required)
- Input checksums: GFF3, FASTA, FAI
- `input_hashes`: `gff3_sha256`, `fasta_sha256`, `fai_sha256`, `policy_sha256`
- Derived checksum: SQLite
- `db_hash` required and must equal `checksums.sqlite_sha256`
- `artifact_hash` required and computed from deterministic manifest fields + `db_hash` (excludes `created_at`)
- Basic stats: gene/transcript/contig counts
- Versions: manifest version + DB schema version
- Dataset signature hash: `dataset_signature_sha256` (Merkle-style over table content)
- Schema evolution note: `schema_evolution_note`
- `toolchain_hash` (rust toolchain + lockfile digest)
- `created_at` allowed for metadata; excluded from determinism signature
- `qc_report_path` points to the emitted QC artifact (canonical: `derived/qc_report.json`)
- `sharding_plan` required (`none|contig|region_grid`) even when `none`
- `contig_normalization_aliases` stores seqid alias mapping used during ingest
- Artifact hash v1 is defined over deterministic SQLite bytes plus stable manifest checksum fields; wall-clock metadata is excluded
- Derived column lineage map: `derived_column_origins`

Unknown fields are rejected.

## What

Defines a stable contract surface for this topic.

## Why

Prevents ambiguity and drift across CLI, API, and operations.

## Scope

Applies to atlas contract consumers and producers.

## Non-goals

Does not define internal implementation details beyond the contract surface.

## Contracts

Use the rules in this page as the normative contract.

## Failure modes

Invalid contract input is rejected with stable machine-readable errors.

## Examples

```bash
$ make ssot-check
```

Expected output: a zero exit code and "contract artifacts generated" for successful checks.

## How to verify

Run `make docs docs-freeze ssot-check` and confirm all commands exit with status 0.

## See also

- [Contracts Overview](../index.md)
- [SSOT Workflow](../ssot-workflow.md)
- [Terms Glossary](../../../glossary.md)
