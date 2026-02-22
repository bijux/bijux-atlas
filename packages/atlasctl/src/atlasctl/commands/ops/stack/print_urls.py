#!/usr/bin/env python3
from __future__ import annotations

import json
import os
from pathlib import Path


def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for parent in cur.parents:
        if all((parent / marker).exists() for marker in ("makefiles", "packages", "configs", "ops")):
            return parent
    raise RuntimeError("unable to resolve repo root")


def _url(name: str, default_port: int) -> str:
    if name == 'atlas':
        return os.environ.get('ATLAS_BASE_URL', 'http://127.0.0.1:18080')
    port_env = {'grafana': 'GRAFANA_PORT', 'prometheus': 'PROMETHEUS_PORT'}[name]
    port = int(os.environ.get(port_env, str(default_port)))
    return f'http://127.0.0.1:{port}'


def main() -> int:
    root = _repo_root()
    out_path = Path(os.environ.get('OPS_RUN_DIR', 'artifacts/ops/manual')) / 'ports.json'
    if not out_path.is_absolute():
        out_path = root / out_path
    out_path.parent.mkdir(parents=True, exist_ok=True)
    payload = {
        'atlas': _url('atlas', 18080),
        'grafana': _url('grafana', 13000),
        'prometheus': _url('prometheus', 19090),
    }
    out_path.write_text(json.dumps(payload, indent=2, sort_keys=True) + '\n', encoding='utf-8')
    print(f"atlas={payload['atlas']}")
    print(f"grafana={payload['grafana']}")
    print(f"prometheus={payload['prometheus']}")
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
