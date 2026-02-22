from __future__ import annotations

import json
from pathlib import Path


def main() -> int:
    cfg = json.loads(Path('configs/ops/observability-pack.json').read_text(encoding='utf-8'))
    ports = cfg['ports']
    print(f"ATLAS_PROM_URL=http://127.0.0.1:{ports['prometheus']}")
    print(f"ATLAS_GRAFANA_URL=http://127.0.0.1:{ports['grafana']}")
    print(f"ATLAS_OTEL_GRPC_ADDR=127.0.0.1:{ports['otel_grpc']}")
    print(f"ATLAS_OTEL_HTTP_URL=http://127.0.0.1:{ports['otel_http']}")
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
