from __future__ import annotations

import json
import os
import shutil
import subprocess
from pathlib import Path


def main() -> int:
    root = Path.cwd()
    out_dir = Path(os.environ.get('OUT_DIR') or (os.sys.argv[1] if len(os.sys.argv) > 1 else 'artifacts/observability/pack-bundle'))
    out_dir.mkdir(parents=True, exist_ok=True)
    copies = [
        ('ops/obs/grafana/atlas-observability-dashboard.json', 'dashboard.json'),
        ('ops/obs/alerts/atlas-alert-rules.yaml', 'alerts.yaml'),
        ('ops/stack/prometheus/prometheus.yaml', 'prometheus.k8s.yaml'),
        ('ops/stack/otel/otel-collector.yaml', 'otel-collector.k8s.yaml'),
        ('ops/obs/pack/compose/docker-compose.yml', 'docker-compose.yml'),
        ('ops/obs/pack/compose/prometheus.yml', 'prometheus.compose.yml'),
        ('ops/obs/pack/compose/otel-collector.yaml', 'otel-collector.compose.yaml'),
        ('configs/ops/observability-pack.json', 'observability-pack.json'),
    ]
    for src, dst in copies:
        shutil.copyfile(root / src, out_dir / dst)
    sha = subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip()
    cfg = json.loads((root / 'configs/ops/observability-pack.json').read_text(encoding='utf-8'))
    payload = {'git_sha': sha, 'schema_version': cfg.get('schema_version')}
    (out_dir / 'pack-version-stamp.json').write_text(
        json.dumps(payload, indent=2, sort_keys=True) + "\n",
        encoding='utf-8',
    )
    print(f'pack bundle exported to {out_dir}')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
