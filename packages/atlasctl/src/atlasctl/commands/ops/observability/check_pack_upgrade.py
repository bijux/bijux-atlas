from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path


def main() -> int:
    profile = os.environ.get('PROFILE', os.environ.get('ATLAS_OBS_PROFILE', 'kind'))
    if len(sys.argv) > 2 and sys.argv[1] == '--profile':
        profile = sys.argv[2]
    cfg = json.loads(Path('configs/ops/observability-pack.json').read_text(encoding='utf-8'))
    required = {
        'prometheus': cfg['images']['prometheus']['ref'],
        'grafana': cfg['images']['grafana']['ref'],
        'otel': cfg['images']['otel_collector']['ref'],
    }
    if profile == 'local-compose':
        out = subprocess.check_output(['docker', 'ps', '--format', '{{.Image}} {{.Names}}'], text=True)
        have = out.splitlines()
        checks = [('prometheus', required['prometheus']), ('grafana', required['grafana']), ('otel-collector', required['otel'])]
        for name, img in checks:
            if not any(img in line and name in line for line in have):
                print(f'local-compose running image mismatch for {name}: expected {img}', file=sys.stderr)
                return 1
    else:
        ns = os.environ.get('ATLAS_OBS_NAMESPACE', 'atlas-observability')
        out = subprocess.check_output(['kubectl', '-n', ns, 'get', 'pods', '-o', 'json'], text=True)
        pods = json.loads(out).get('items', [])
        found = {c.get('image', '') for p in pods for c in p.get('spec', {}).get('containers', [])}
        for img in required.values():
            if img not in found:
                print(f'k8s running image mismatch: expected {img}', file=sys.stderr)
                return 1
    print('observability pack upgrade check passed')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
