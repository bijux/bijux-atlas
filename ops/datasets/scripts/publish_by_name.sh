#!/usr/bin/env bash
# owner: bijux-atlas-operations
# purpose: publish named dataset via ingest + catalog publish.
# stability: public
# called-by: make ops-publish
set -euo pipefail
ROOT="$(CDPATH='' cd -- "$(dirname -- "$0")/../../.." && pwd)"
DATASET="${1:-medium}"
case "$DATASET" in
  medium)
    "$ROOT/ops/e2e/scripts/publish_dataset.sh" \
      --gff3 "$ROOT/ops/fixtures/medium/data/genes.gff3" \
      --fasta "$ROOT/ops/fixtures/medium/data/genome.fa" \
      --fai "$ROOT/ops/fixtures/medium/data/genome.fa.fai" \
      --release 110 --species homo_sapiens --assembly GRCh38
    ;;
  real1)
    "$ROOT/scripts/fixtures/fetch-real-datasets.sh" >/dev/null
    "$ROOT/ops/e2e/scripts/publish_dataset.sh" \
      --gff3 "$ROOT/artifacts/real-datasets/111/homo_sapiens/GRCh38/genes.gff3" \
      --fasta "$ROOT/artifacts/real-datasets/111/homo_sapiens/GRCh38/genome.fa" \
      --fai "$ROOT/artifacts/real-datasets/111/homo_sapiens/GRCh38/genome.fa.fai" \
      --release 111 --species homo_sapiens --assembly GRCh38
    ;;
  *)
    echo "unsupported DATASET=$DATASET (expected: medium|real1)" >&2
    exit 2
    ;;
esac
