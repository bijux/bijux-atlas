from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[7]
MANIFESTS = [
    ROOT / 'ops' / 'e2e' / 'suites' / 'suites.json',
]


def _cmd_ok(cmd: list[object]) -> bool:
    if not cmd:
        return False
    parts = [str(x) for x in cmd]
    text = ' '.join(parts)
    forbidden = {'bash', 'sh', '-lc'}
    if any(tok in forbidden for tok in parts):
        return False
    if ' bash ' in f' {text} ' or ' -lc ' in f' {text} ':
        return False
    head = parts[0]
    if head == './bin/atlasctl':
        return True
    if head == 'python3' and len(parts) >= 2 and parts[1].startswith('packages/atlasctl/src/atlasctl/'):
        return True
    return False


def main() -> int:
    errors: list[str] = []
    for path in MANIFESTS:
        if not path.exists():
            continue
        payload = json.loads(path.read_text(encoding='utf-8'))
        suites = payload.get('suites', [])
        for idx, row in enumerate(suites):
            if not isinstance(row, dict):
                continue
            runner = row.get('runner')
            if isinstance(runner, list):
                if not _cmd_ok(runner):
                    errors.append(f"{path.relative_to(ROOT)}:suites[{idx}] runner must be atlasctl-native (no raw bash/sh): {runner}")
        scenarios = payload.get('scenarios', {})
        if isinstance(scenarios, dict):
            for name, row in sorted(scenarios.items()):
                if not isinstance(row, dict):
                    continue
                cmd = row.get('command')
                if isinstance(cmd, list) and not _cmd_ok(cmd):
                    errors.append(f"{path.relative_to(ROOT)}:scenario `{name}` command must be atlasctl-native (no raw bash/sh): {cmd}")
    if errors:
        for err in errors:
            print(err, file=sys.stderr)
        return 1
    print('ops scenario manifests reference atlasctl-native commands only')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
