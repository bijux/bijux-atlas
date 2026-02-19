#!/usr/bin/env bash
# owner: bijux-atlas-operations
# purpose: publish named dataset via ingest + catalog publish.
# stability: public
# called-by: make ops-publish
set -euo pipefail
ROOT="$(CDPATH='' cd -- "$(dirname -- "$0")/../../.." && pwd)"
DATASET="${1:-medium}"
REAL_ROOT="${ATLAS_REALDATA_ROOT:-$ROOT/artifacts/real-datasets}"
case "$DATASET" in
  medium)
    "$ROOT/ops/e2e/runner/publish_dataset.sh" \
      --gff3 "$ROOT/ops/fixtures/medium/v1/data/genes.gff3" \
      --fasta "$ROOT/ops/fixtures/medium/v1/data/genome.fa" \
      --fai "$ROOT/ops/fixtures/medium/v1/data/genome.fa.fai" \
      --release 110 --species homo_sapiens --assembly GRCh38
    ;;
  real1)
    "$ROOT/scripts/fixtures/fetch-real-datasets.sh" >/dev/null
    "$ROOT/ops/e2e/runner/publish_dataset.sh" \
      --gff3 "$REAL_ROOT/111/homo_sapiens/GRCh38/genes.gff3" \
      --fasta "$REAL_ROOT/111/homo_sapiens/GRCh38/genome.fa" \
      --fai "$REAL_ROOT/111/homo_sapiens/GRCh38/genome.fa.fai" \
      --release 111 --species homo_sapiens --assembly GRCh38
    ;;
  real110)
    "$ROOT/scripts/fixtures/fetch-real-datasets.sh" >/dev/null
    "$ROOT/ops/e2e/runner/publish_dataset.sh" \
      --gff3 "$REAL_ROOT/110/homo_sapiens/GRCh38/genes.gff3" \
      --fasta "$REAL_ROOT/110/homo_sapiens/GRCh38/genome.fa" \
      --fai "$REAL_ROOT/110/homo_sapiens/GRCh38/genome.fa.fai" \
      --release 110 --species homo_sapiens --assembly GRCh38
    ;;
  real111)
    "$ROOT/scripts/fixtures/fetch-real-datasets.sh" >/dev/null
    "$ROOT/ops/e2e/runner/publish_dataset.sh" \
      --gff3 "$REAL_ROOT/111/homo_sapiens/GRCh38/genes.gff3" \
      --fasta "$REAL_ROOT/111/homo_sapiens/GRCh38/genome.fa" \
      --fai "$REAL_ROOT/111/homo_sapiens/GRCh38/genome.fa.fai" \
      --release 111 --species homo_sapiens --assembly GRCh38
    ;;
  *)
    echo "unsupported DATASET=$DATASET (expected: medium|real1|real110|real111)" >&2
    exit 2
    ;;
esac
