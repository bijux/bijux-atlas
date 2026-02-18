#!/usr/bin/env bash
set -euo pipefail

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
RUN_ID="${OPS_RUN_ID:-atlas-incident-$(date -u +%Y%m%d-%H%M%S)}"
OUT="$ROOT/artifacts/incident/$RUN_ID"
mkdir -p "$OUT"

python3 "$ROOT/scripts/public/config-print.py" > "$OUT/config.print.json"
cp -f "$ROOT/ops/_schemas/report/schema.json" "$OUT/ops-report-schema.json" 2>/dev/null || true
cp -f "$ROOT/artifacts/ops/$RUN_ID/metadata.json" "$OUT/metadata.json" 2>/dev/null || true
cp -f "$ROOT/artifacts/ops/$RUN_ID/report.json" "$OUT/report.json" 2>/dev/null || true
cp -f "$ROOT/artifacts/ops/obs/metrics.prom" "$OUT/metrics.prom" 2>/dev/null || true
cp -f "$ROOT/artifacts/ops/obs/traces.snapshot.log" "$OUT/traces.snapshot.log" 2>/dev/null || true
cp -f "$ROOT/artifacts/ops/obs/traces.exemplars.log" "$OUT/traces.exemplars.log" 2>/dev/null || true

find "$ROOT/artifacts/ops" -type f \( -name "*.log" -o -name "*.json" -o -name "*.md" \) | tail -n 200 > "$OUT/artifact-file-list.txt" || true

echo "$OUT"
