#!/usr/bin/env bash
# owner: bijux-atlas-operations
# purpose: run QC validation gate on minimal and medium fixtures.
# stability: public
# called-by: make ci-qc-fixtures
set -euo pipefail

ROOT="$(CDPATH='' cd -- "$(dirname -- "$0")/../.." && pwd)"
OUT_ROOT="${ROOT}/artifacts/isolates/qc-fixtures"
THRESHOLDS="${ROOT}/configs/ops/dataset-qc-thresholds.json"
rm -rf "$OUT_ROOT"
mkdir -p "$OUT_ROOT"

run_case() {
  local case_name="$1"
  local gff3="$2"
  local fasta="$3"
  local fai="$4"
  local release="$5"
  local species="$6"
  local assembly="$7"
  local case_out="${OUT_ROOT}/${case_name}"
  mkdir -p "$case_out"
  cargo run -q -p bijux-atlas-cli --bin bijux-atlas -- atlas ingest \
    --gff3 "$gff3" \
    --fasta "$fasta" \
    --fai "$fai" \
    --output-root "$case_out" \
    --release "$release" \
    --species "$species" \
    --assembly "$assembly" \
    --strictness strict \
    --max-threads 1
  local qc="${case_out}/release=${release}/species=${species}/assembly=${assembly}/derived/qc.json"
  cargo run -q -p bijux-atlas-cli --bin bijux-atlas -- atlas ingest-validate \
    --qc-report "$qc" \
    --thresholds "$THRESHOLDS"
}

run_case \
  minimal \
  "${ROOT}/crates/bijux-atlas-ingest/tests/fixtures/tiny/genes.gff3" \
  "${ROOT}/crates/bijux-atlas-ingest/tests/fixtures/tiny/genome.fa" \
  "${ROOT}/crates/bijux-atlas-ingest/tests/fixtures/tiny/genome.fa.fai" \
  110 homo_sapiens GRCh38

run_case \
  medium \
  "${ROOT}/ops/fixtures/medium/data/genes.gff3" \
  "${ROOT}/ops/fixtures/medium/data/genome.fa" \
  "${ROOT}/ops/fixtures/medium/data/genome.fa.fai" \
  110 homo_sapiens GRCh38

echo "qc fixtures gate passed"
