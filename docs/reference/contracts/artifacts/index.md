# Artifact Schema Contract

Owner: `docs-governance`  
Type: `reference`  
Audience: `contributor`  
Reason to exist: define the canonical artifact layout and manifest/schema constraints.

## Directory Layout

Canonical dataset directory:

`release=<release>/species=<species>/assembly=<assembly>/`

Required artifacts:

- `inputs/genes.gff3.bgz`
- `inputs/genome.fa.bgz`
- `inputs/genome.fa.bgz.fai`
- `derived/gene_summary.sqlite`
- `derived/manifest.json`
- `derived/release_gene_index.json`

Optional diff artifacts:

- `derived/diffs/from=<from_release>/to=<to_release>/diff.json`
- `derived/diffs/from=<from_release>/to=<to_release>/diff.summary.json`
- `derived/diffs/from=<from_release>/to=<to_release>/chunks/*.json`

## Manifest Contract

Manifest is strict JSON and includes:

- dataset identity (`release`, `species`, `assembly`)
- schema/version metadata (`schema_version`, `db_schema_version`, `artifact_version`)
- input hashes (`gff3_sha256`, `fasta_sha256`, `fai_sha256`, `policy_sha256`)
- SQLite and artifact integrity (`db_hash`, `checksums.sqlite_sha256`, `artifact_hash`)

## SQLite Contract

- SSOT schema: `crates/bijux-atlas-ingest/sql/schema_v4.sql`
- Read-only serving expectations: immutable open mode and query-only behavior
- Forward-only schema upgrades with compatibility gates

## Schema Source

- `docs/reference/contracts/schemas/ARTIFACT_SCHEMA.json`
