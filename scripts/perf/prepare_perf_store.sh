#!/usr/bin/env sh
# Purpose: script interface entrypoint.
# Inputs: command-line args and repository files/env as documented by caller.
# Outputs: exit status and deterministic stdout/stderr or generated artifacts.
set -eu

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
OUT="${1:-$ROOT/artifacts/perf/store}"
TMP="${ROOT}/artifacts/perf/_tmp"
DATASET_RELEASE="110"
DATASET_SPECIES="homo_sapiens"
DATASET_ASSEMBLY="GRCh38"

rm -rf "$OUT" "$TMP"
mkdir -p "$OUT" "$TMP"

cargo run --quiet -p bijux-atlas-cli --bin bijux-atlas -- atlas ingest \
  --gff3 "$ROOT/crates/bijux-atlas-ingest/tests/ops/fixtures/tiny/genes.gff3" \
  --fasta "$ROOT/crates/bijux-atlas-ingest/tests/ops/fixtures/tiny/genome.fa" \
  --fai "$ROOT/crates/bijux-atlas-ingest/tests/ops/fixtures/tiny/genome.fa.fai" \
  --output-root "$OUT" \
  --release "$DATASET_RELEASE" \
  --species "$DATASET_SPECIES" \
  --assembly "$DATASET_ASSEMBLY" >/dev/null

cat > "$OUT/catalog.json" <<JSON
{
  "datasets": [
    {
      "dataset": {
        "release": "$DATASET_RELEASE",
        "species": "$DATASET_SPECIES",
        "assembly": "$DATASET_ASSEMBLY"
      },
      "manifest_path": "$DATASET_RELEASE/$DATASET_SPECIES/$DATASET_ASSEMBLY/derived/manifest.json",
      "sqlite_path": "$DATASET_RELEASE/$DATASET_SPECIES/$DATASET_ASSEMBLY/derived/gene_summary.sqlite"
    }
  ]
}
JSON

echo "prepared perf store at $OUT"