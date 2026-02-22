#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../../../../../../../../../.." && pwd)"
ART_DIR="$ROOT/artifacts/ops/obs/tests"
mkdir -p "$ART_DIR"

require_bin() {
  command -v "$1" >/dev/null 2>&1 || { echo "missing required binary: $1" >&2; exit 2; }
}

capture_failure_artifacts() {
  local out="${1:-$ART_DIR/failure-$(date +%Y%m%d-%H%M%S)}"
  mkdir -p "$out"
  kubectl get pods -A -o wide > "$out/pods.txt" 2>/dev/null || true
  kubectl get events -A --sort-by=.lastTimestamp > "$out/events.txt" 2>/dev/null || true
  kubectl logs -n "${ATLAS_NS:-atlas-e2e}" -l app.kubernetes.io/name=bijux-atlas --tail=2000 > "$out/atlas.log" 2>/dev/null || true
  cp -f "$ROOT/ops/obs/grafana/atlas-observability-dashboard.json" "$out/dashboard.json" 2>/dev/null || true
  cp -f "$ROOT/ops/obs/alerts/atlas-alert-rules.yaml" "$out/alerts.yaml" 2>/dev/null || true
  echo "$out"
}
