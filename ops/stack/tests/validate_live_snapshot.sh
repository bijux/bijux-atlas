#!/usr/bin/env bash
set -euo pipefail

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
# shellcheck source=/dev/null
. "$ROOT/ops/_lib/common.sh"

NS="${ATLAS_E2E_NAMESPACE:-$(ops_layer_ns_stack)}"
OUT="${1:-$ROOT/ops/_generated/contracts/live-snapshot.json}"
mkdir -p "$(dirname "$OUT")"

if ! kubectl cluster-info >/dev/null 2>&1; then
  echo "live snapshot validation skipped: cluster not available"
  exit 0
fi
if ! kubectl get ns "$NS" >/dev/null 2>&1; then
  echo "live snapshot validation skipped: namespace $NS not found"
  exit 0
fi

kubectl -n "$NS" get svc -o json > "$OUT"
python3 - "$OUT" "$ROOT/ops/_meta/layer-contract.json" <<'PY'
import json
import sys

snapshot = json.load(open(sys.argv[1], encoding="utf-8"))
contract = json.load(open(sys.argv[2], encoding="utf-8"))
svc_map = {item["metadata"]["name"]: item for item in snapshot.get("items", [])}

for comp, cfg in contract["services"].items():
    name = cfg["service_name"]
    if name not in svc_map:
        # stack-only snapshots may not include atlas or optional components.
        continue

for comp, ports in contract["ports"].items():
    svc_name = contract["services"][comp]["service_name"]
    if svc_name not in svc_map:
        continue
    live_ports = {p.get("name") or "service": int(p["port"]) for p in svc_map[svc_name]["spec"].get("ports", [])}
    for key, val in ports.items():
        if key in ("container",):
            continue
        if key in live_ports and live_ports[key] != int(val):
            raise SystemExit(f"port mismatch for {svc_name}:{key}; expected={val} live={live_ports[key]}")
        if key not in live_ports and key == "service":
            first = next(iter(live_ports.values()), None)
            if first is not None and first != int(val):
                raise SystemExit(f"port mismatch for {svc_name}; expected={val} live={first}")
PY

echo "live layer contract validation passed"
