#!/usr/bin/env bash
# owner: bijux-atlas-operations
# purpose: enforce dataset QC thresholds from config.
# stability: public
# called-by: make ops-dataset-qc
set -euo pipefail
ROOT="$(CDPATH='' cd -- "$(dirname -- "$0")/../../../.." && pwd)"
QC_CFG="${ATLAS_QC_THRESHOLDS:-$ROOT/configs/ops/dataset-qc-thresholds.v1.json}"
REPORT_DIR="${ATLAS_E2E_OUTPUT_ROOT:-$ROOT/artifacts/e2e-datasets}"
release="${ATLAS_DATASET_RELEASE:-110}"
species="${ATLAS_DATASET_SPECIES:-homo_sapiens}"
assembly="${ATLAS_DATASET_ASSEMBLY:-GRCh38}"
QC_REPORT="$REPORT_DIR/release=$release/species=$species/assembly=$assembly/derived/qc.json"
if [ ! -f "$QC_REPORT" ]; then
  echo "dataset QC failed: missing qc report: $QC_REPORT" >&2
  exit 1
fi
cargo run -q -p bijux-atlas-cli --bin bijux-atlas -- atlas ingest-validate \
  --qc-report "$QC_REPORT" \
  --thresholds "$QC_CFG"
python3 "$ROOT/packages/atlasctl/src/atlasctl/commands/ops/datasets/qc_summary.py" \
  --qc "$QC_REPORT" \
  --out "$REPORT_DIR/qc-summary.md"
