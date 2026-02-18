#!/usr/bin/env bash
set -euo pipefail
# shellcheck source=ops/_lib/common.sh
source "$(CDPATH= cd -- "$(dirname -- "$0")/../../_lib" && pwd)/common.sh"

OUT_DIR="${1:-${REPO_ROOT}/artifacts/observability/pack-bundle}"
mkdir -p "$OUT_DIR"

cp -f "${REPO_ROOT}/ops/obs/grafana/atlas-observability-dashboard.json" "$OUT_DIR/dashboard.json"
cp -f "${REPO_ROOT}/ops/obs/alerts/atlas-alert-rules.yaml" "$OUT_DIR/alerts.yaml"
cp -f "${REPO_ROOT}/ops/stack/prometheus/prometheus.yaml" "$OUT_DIR/prometheus.k8s.yaml"
cp -f "${REPO_ROOT}/ops/stack/otel/otel-collector.yaml" "$OUT_DIR/otel-collector.k8s.yaml"
cp -f "${REPO_ROOT}/ops/obs/pack/compose/docker-compose.yml" "$OUT_DIR/docker-compose.yml"
cp -f "${REPO_ROOT}/ops/obs/pack/compose/prometheus.yml" "$OUT_DIR/prometheus.compose.yml"
cp -f "${REPO_ROOT}/ops/obs/pack/compose/otel-collector.yaml" "$OUT_DIR/otel-collector.compose.yaml"
cp -f "${REPO_ROOT}/configs/ops/observability-pack.json" "$OUT_DIR/observability-pack.json"

PACK_BUNDLE_OUT="$OUT_DIR" python3 - <<'PY'
import json,subprocess
import os
from pathlib import Path
out=Path(os.environ["PACK_BUNDLE_OUT"])
sha=subprocess.check_output(["git","rev-parse","HEAD"],text=True).strip()
cfg=json.loads(Path("configs/ops/observability-pack.json").read_text(encoding="utf-8"))
payload={"git_sha":sha,"schema_version":cfg.get("schema_version")}
(out/"pack-version-stamp.json").write_text(json.dumps(payload,indent=2,sort_keys=True)+"\n",encoding="utf-8")
PY

echo "pack bundle exported to $OUT_DIR"
