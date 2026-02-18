#!/usr/bin/env bash
set -euo pipefail
ATLAS_BASE_URL="${ATLAS_BASE_URL:-http://127.0.0.1:18080}"
OUT_DIR="${OUT_DIR:-artifacts/ops/observability/drills}"
mkdir -p "$OUT_DIR"

before_file="$OUT_DIR/memory-before.metrics"
after_file="$OUT_DIR/memory-after.metrics"
report_file="$OUT_DIR/memory-growth-report.md"

curl -fsS "$ATLAS_BASE_URL/metrics" > "$before_file"
sleep 1
curl -fsS "$ATLAS_BASE_URL/metrics" > "$after_file"

before="$(awk '$1=="process_resident_memory_bytes" {print $2; exit}' "$before_file")"
after="$(awk '$1=="process_resident_memory_bytes" {print $2; exit}' "$after_file")"
[ -n "${before:-}" ] && [ -n "${after:-}" ]
growth=$(( ${after%.*} - ${before%.*} ))

cat > "$report_file" <<EOF
# Memory Growth Drill Report

- before_bytes: ${before}
- after_bytes: ${after}
- growth_bytes: ${growth}
EOF

echo "memory growth drill assertions passed"
echo "report: $report_file"
