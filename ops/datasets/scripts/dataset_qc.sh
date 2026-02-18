#!/usr/bin/env bash
# owner: bijux-atlas-operations
# purpose: enforce dataset QC thresholds from config.
# stability: public
# called-by: make ops-dataset-qc
set -euo pipefail
ROOT="$(CDPATH='' cd -- "$(dirname -- "$0")/../../.." && pwd)"
QC_CFG="$ROOT/configs/ops/dataset-qc-thresholds.json"
REPORT_DIR="${ATLAS_E2E_OUTPUT_ROOT:-$ROOT/artifacts/e2e-datasets}"
python3 - <<'PY'
from pathlib import Path
import json,sys
root=Path.cwd()
qc=json.loads((root/'configs/ops/dataset-qc-thresholds.json').read_text())
max_warn=qc['max_qc_warn']; max_err=qc['max_qc_error']; max_anom=qc['max_anomalies']
qc_report=root/'artifacts/e2e-datasets/qc_report.json'
anom_report=root/'artifacts/e2e-datasets/anomalies.json'
warn=err=anom=0
if qc_report.exists():
    d=json.loads(qc_report.read_text())
    warn=len(d.get('warnings',[])); err=len(d.get('errors',[]))
if anom_report.exists():
    d=json.loads(anom_report.read_text())
    anom=len(d.get('anomalies',[]))
if warn>max_warn or err>max_err or anom>max_anom:
    print(f"dataset QC failed: warnings={warn}/{max_warn} errors={err}/{max_err} anomalies={anom}/{max_anom}", file=sys.stderr)
    raise SystemExit(1)
print(f"dataset QC passed: warnings={warn} errors={err} anomalies={anom}")
PY
