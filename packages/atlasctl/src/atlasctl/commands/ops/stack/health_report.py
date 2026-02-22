#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
import subprocess
from datetime import datetime, timezone
from pathlib import Path


def _run(*cmd: str) -> dict[str, object]:
    p = subprocess.run(list(cmd), capture_output=True, text=True, check=False)
    return {"ok": p.returncode == 0, "code": p.returncode, "stdout": p.stdout.strip(), "stderr": p.stderr.strip()}


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument('namespace', nargs='?', default='atlas-e2e')
    ap.add_argument('out', nargs='?', default='artifacts/ops/stack/health-report.txt')
    args = ap.parse_args()
    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    fmt = os.environ.get('ATLAS_HEALTH_REPORT_FORMAT', 'text')
    ns = args.namespace
    if fmt == 'json':
        payload = {
            'schema_version': 1,
            'namespace': ns,
            'timestamp': datetime.now(timezone.utc).isoformat(),
            'checks': {
                'nodes': _run('kubectl', 'get', 'nodes', '-o', 'name'),
                'pods': _run('kubectl', '-n', ns, 'get', 'pods', '-o', 'name'),
                'services': _run('kubectl', '-n', ns, 'get', 'svc', '-o', 'name'),
                'storageclass': _run('kubectl', 'get', 'storageclass', '-o', 'name'),
            },
        }
        out.write_text(json.dumps(payload, indent=2, sort_keys=True) + '\n', encoding='utf-8')
    else:
        parts = [f'namespace={ns}', f'timestamp={datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")}', '--- nodes ---']
        for cmd in (
            ['kubectl', 'get', 'nodes', '-o', 'wide'],
            ['kubectl', '-n', ns, 'get', 'pods', '-o', 'wide'],
            ['kubectl', '-n', ns, 'get', 'svc'],
            ['kubectl', 'get', 'storageclass'],
        ):
            if cmd[2] == 'nodes':
                pass
        labels = [
            ('--- nodes ---', ['kubectl', 'get', 'nodes', '-o', 'wide']),
            ('--- pods ---', ['kubectl', '-n', ns, 'get', 'pods', '-o', 'wide']),
            ('--- services ---', ['kubectl', '-n', ns, 'get', 'svc']),
            ('--- storageclass ---', ['kubectl', 'get', 'storageclass']),
        ]
        lines: list[str] = [f'namespace={ns}', f'timestamp={datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")}']
        for header, cmd in labels:
            lines.append(header)
            p = subprocess.run(cmd, capture_output=True, text=True, check=False)
            if p.stdout:
                lines.extend(p.stdout.rstrip().splitlines())
        out.write_text('\n'.join(lines) + '\n', encoding='utf-8')
    print(out)
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
