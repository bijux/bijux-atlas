# Ensembl Layout Ingest Workflow

Typical mapping:
- Ensembl GFF3 -> `--gff3`
- Ensembl genome FASTA -> `--fasta`
- Ensembl FASTA index (`.fai`) -> `--fai`

Flow:
1. Validate dataset dimensions (`release/species/assembly`).
2. Run ingest with explicit strictness and identifier policy.
3. Publish produced artifacts into store backend only after checksum verification.

Output contract aligns with root artifact docs:
- `docs/artifact-directory-contract.md`
- `docs/artifact-manifest-contract.md`
