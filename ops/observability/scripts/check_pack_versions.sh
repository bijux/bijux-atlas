#!/usr/bin/env bash
set -euo pipefail
# shellcheck source=ops/_lib/common.sh
source "$(CDPATH= cd -- "$(dirname -- "$0")/../../_lib" && pwd)/common.sh"

python3 - <<'PY'
import json,re,sys
from pathlib import Path

cfg=json.loads(Path("configs/ops/observability-pack.json").read_text(encoding="utf-8"))
images=cfg.get("images",{})
if not images:
    print("configs/ops/observability-pack.json: images missing",file=sys.stderr)
    raise SystemExit(1)

digest_re=re.compile(r"^sha256:[a-fA-F0-9]{64}$")
for name,spec in images.items():
    ref=spec.get("ref","")
    digest=spec.get("digest","")
    if ":latest" in ref or not ref:
        print(f"{name}: image ref must be pinned tag (not latest): {ref}",file=sys.stderr)
        raise SystemExit(1)
    if not digest_re.match(digest):
        print(f"{name}: digest missing/invalid: {digest}",file=sys.stderr)
        raise SystemExit(1)

compose=Path("ops/observability/pack/compose/docker-compose.yml").read_text(encoding="utf-8")
for name,spec in images.items():
    if spec["ref"] not in compose:
        print(f"compose missing expected image ref for {name}: {spec['ref']}",file=sys.stderr)
        raise SystemExit(1)

required_services = ["prometheus", "grafana", "otel-collector"]
for svc in required_services:
    if f"{svc}:" not in compose:
        print(f"compose missing required service: {svc}", file=sys.stderr)
        raise SystemExit(1)

ports = cfg.get("ports", {})
required_port_bindings = {
    "prometheus": f"{ports.get('prometheus')}:9090",
    "grafana": f"{ports.get('grafana')}:3000",
    "otel_grpc": f"{ports.get('otel_grpc')}:4317",
    "otel_http": f"{ports.get('otel_http')}:4318",
}
for key, binding in required_port_bindings.items():
    if binding not in compose:
        print(f"compose missing required port mapping for {key}: {binding}", file=sys.stderr)
        raise SystemExit(1)

print("observability pack version/digest policy check passed")
PY
