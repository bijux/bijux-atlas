#!/usr/bin/env python3
from __future__ import annotations

import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[7]


def main() -> int:
    script = r"""
set -euo pipefail
ROOT="$(pwd)"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

cat > "$TMP/metrics.prom" <<'EOF'
bijux_http_requests_total{route="/v1/genes",status="200"} 990
bijux_http_requests_total{route="/v1/genes",status="500"} 10
bijux_store_breaker_open 0
EOF
cat > "$TMP/score.md" <<'EOF'
K6 Summary
Checks 100.00%
EOF

python3 "$ROOT/packages/atlasctl/src/atlasctl/commands/ops/observability/compute_slo_burn.py" --k6 "$TMP/score.md" --metrics "$TMP/metrics.prom" --out "$TMP/out-a.json" >/dev/null
python3 "$ROOT/packages/atlasctl/src/atlasctl/commands/ops/observability/compute_slo_burn.py" --k6 "$TMP/score.md" --metrics "$TMP/metrics.prom" --out "$TMP/out-b.json" >/dev/null
cmp -s "$TMP/out-a.json" "$TMP/out-b.json"

echo "slo burn determinism passed"
"""
    return subprocess.run(["bash", "-lc", script], cwd=ROOT).returncode


if __name__ == "__main__":
    raise SystemExit(main())
