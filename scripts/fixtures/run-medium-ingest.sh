#!/usr/bin/env sh
set -eu

root="fixtures/medium/data"
[ -d "$root" ] || { echo "missing medium fixture data; run make fetch-fixtures" >&2; exit 1; }

cargo run -p bijux-atlas-cli --bin bijux-atlas -- atlas ingest \
  --gff3 "$root/genes.gff3" \
  --fasta "$root/genome.fa" \
  --fai "$root/genome.fa.fai" \
  --output-root artifacts/medium-output \
  --release 110 \
  --species homo_sapiens \
  --assembly GRCh38 \
  --strictness lenient \
  --duplicate-gene-id-policy dedupe
