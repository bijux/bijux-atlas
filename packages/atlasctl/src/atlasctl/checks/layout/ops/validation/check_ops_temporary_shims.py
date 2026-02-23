from __future__ import annotations

import datetime as dt
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[8]
CFG = ROOT / 'configs' / 'ops' / 'temporary-shims.json'
PARSER = ROOT / 'packages' / 'atlasctl' / 'src' / 'atlasctl' / 'commands' / 'ops' / 'runtime_modules' / 'ops_runtime_parser.py'
_APPROVAL_RE = re.compile(r'^OPS-SHIM-[0-9]{3,}$')


def _discover_runtime_shims() -> set[str]:
    text = PARSER.read_text(encoding='utf-8', errors='ignore')
    discovered: set[str] = set()
    if 'run-script' in text and 'migration shim' in text:
        discovered.add('atlasctl ops run-script')
    if 'root-lanes' in text and 'migration shim' in text:
        discovered.add('atlasctl ops root-lanes')
    if 'root-local' in text and 'migration shim' in text:
        discovered.add('atlasctl ops root-local')
    return discovered


def main() -> int:
    payload = json.loads(CFG.read_text(encoding='utf-8'))
    errs: list[str] = []
    if int(payload.get('schema_version', 0) or 0) != 1:
        errs.append('schema_version must be 1')
    rows = payload.get('shims', [])
    if not isinstance(rows, list):
        errs.append('shims must be a list')
        rows = []
    by_cmd: dict[str, dict[str, object]] = {}
    today = dt.date.today()
    for idx, row in enumerate(rows):
        if not isinstance(row, dict):
            errs.append(f'shims[{idx}] must be an object')
            continue
        cmd = str(row.get('command', '')).strip()
        if not cmd:
            errs.append(f'shims[{idx}].command is required')
            continue
        if cmd in by_cmd:
            errs.append(f'duplicate shim command entry: {cmd}')
        by_cmd[cmd] = row
        approval = str(row.get('approval_id', '')).strip()
        if not _APPROVAL_RE.match(approval):
            errs.append(f'{cmd}: invalid or missing approval_id ({approval!r})')
        owner = str(row.get('owner', '')).strip()
        reason = str(row.get('reason', '')).strip()
        if not owner:
            errs.append(f'{cmd}: owner is required')
        if not reason:
            errs.append(f'{cmd}: reason is required')
        expiry_raw = str(row.get('expires_on', '')).strip()
        try:
            expiry = dt.date.fromisoformat(expiry_raw)
        except ValueError:
            errs.append(f'{cmd}: expires_on must be YYYY-MM-DD')
            continue
        if expiry < today:
            errs.append(f'{cmd}: expired shim approval ({expiry_raw})')
    discovered = _discover_runtime_shims()
    configured = set(by_cmd)
    for missing in sorted(discovered - configured):
        errs.append(f'new temporary shim requires approval entry: {missing}')
    for stale in sorted(configured - discovered):
        errs.append(f'shim approval entry points to missing shim: {stale}')
    if errs:
        print('ops temporary shims check failed:', file=sys.stderr)
        for e in errs:
            print(e, file=sys.stderr)
        return 1
    print('ops temporary shims check passed')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
