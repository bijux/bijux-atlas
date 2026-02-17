#!/usr/bin/env sh
set -eu

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
REAL_ROOT="${ATLAS_REALDATA_ROOT:-$ROOT/artifacts/real-datasets}"

"$ROOT/scripts/fixtures/fetch-real-datasets.sh"
"$ROOT/e2e/scripts/cleanup_store.sh"

"$ROOT/e2e/scripts/publish_dataset.sh" \
  --gff3 "$REAL_ROOT/110/homo_sapiens/GRCh38/genes.gff3" \
  --fasta "$REAL_ROOT/110/homo_sapiens/GRCh38/genome.fa" \
  --fai "$REAL_ROOT/110/homo_sapiens/GRCh38/genome.fa.fai" \
  --release 110 --species homo_sapiens --assembly GRCh38

"$ROOT/e2e/scripts/deploy_atlas.sh"
"$ROOT/e2e/scripts/warmup.sh"
"$ROOT/e2e/scripts/smoke_queries.sh"
"$ROOT/e2e/realdata/verify_snapshots.sh"

echo "realdata single-release scenario passed"
