#!/usr/bin/env sh
# Purpose: script interface entrypoint.
# Inputs: command-line args and repository files/env as documented by caller.
# Outputs: exit status and deterministic stdout/stderr or generated artifacts.
set -eu

root="ops/fixtures/medium/v1/data"
[ -d "$root" ] || { echo "missing medium fixture data; run make fetch-fixtures" >&2; exit 1; }
sharded=0
if [ "${1:-}" = "--sharded" ]; then
  sharded=1
fi

if [ "$sharded" -eq 1 ]; then
  cargo run -p bijux-atlas-cli --bin bijux-atlas -- atlas ingest \
    --gff3 "$root/genes.gff3" \
    --fasta "$root/genome.fa" \
    --fai "$root/genome.fa.fai" \
    --output-root artifacts/medium-output \
    --release 110 \
    --species homo_sapiens \
    --assembly GRCh38 \
    --strictness lenient \
    --duplicate-gene-id-policy dedupe \
    --sharding-plan contig \
    --emit-shards
else
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
fi
