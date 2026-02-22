#!/usr/bin/env bash
# owner: bijux-atlas-operations
# purpose: inject high-cardinality metric series and assert guard check rejects it.
# stability: public
# called-by: make observability-pack-drills
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
out="${ROOT}/artifacts/observability/drills/metrics-cardinality.prom"
mkdir -p "$(dirname "$out")"
cat > "$out" <<'EOF'
bijux_http_requests_total{subsystem="atlas",route="/v1/genes",status="200",query_type="list",dataset="d1",version="v1",request_id="r1"} 1
EOF
if python3 "$ROOT/packages/atlasctl/src/atlasctl/commands/ops/observability/check_metric_cardinality.py" "$out" >/dev/null 2>&1; then
  echo "expected metric cardinality check to fail" >&2
  exit 1
fi
echo "metric cardinality blowup attempt drill passed"
