# Artifact Manifest Contract

- Owner: `docs-governance`

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

- [Contracts Overview](../INDEX.md)
- [SSOT Workflow](../ssot-workflow.md)
- [Terms Glossary](../../_style/terms-glossary.md)
