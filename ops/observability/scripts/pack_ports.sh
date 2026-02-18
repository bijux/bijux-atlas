#!/usr/bin/env bash
# owner: bijux-atlas-operations
# purpose: print deterministic observability pack port/url env values from SSOT config.
# stability: internal
# called-by: make ops-observability-pack-verify, make ops-observability-pack-health
set -euo pipefail
python3 - <<'PY'
import json
cfg=json.load(open('configs/ops/observability-pack.json'))
ports=cfg['ports']
print(f"ATLAS_PROM_URL=http://127.0.0.1:{ports['prometheus']}")
print(f"ATLAS_GRAFANA_URL=http://127.0.0.1:{ports['grafana']}")
print(f"ATLAS_OTEL_GRPC_ADDR=127.0.0.1:{ports['otel_grpc']}")
print(f"ATLAS_OTEL_HTTP_URL=http://127.0.0.1:{ports['otel_http']}")
PY
