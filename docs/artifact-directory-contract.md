# Artifact Directory Contract (SSOT)

Canonical layout:

`release=<release>/species=<species>/assembly=<assembly>/`

Inside dataset directory:

- `inputs/genes.gff3.bgz`
- `inputs/genome.fa.bgz`
- `inputs/genome.fa.bgz.fai`
- `derived/gene_summary.sqlite`
- `derived/manifest.json`

This layout is encoded in `bijux-atlas-model::artifact_paths`.
