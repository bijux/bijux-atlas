from __future__ import annotations
import os, subprocess, sys
from pathlib import Path

def main() -> int:
    root = Path.cwd(); out = root / 'artifacts/ops/obs'
    out.mkdir(parents=True, exist_ok=True)
    (out/'traces.snapshot.log').write_text('{"spans":[{"name":"request_root","request_id":"abc"}]}\n', encoding='utf-8')
    (out/'traces.exemplars.log').write_text('{"trace_id":"abc"}\n', encoding='utf-8')
    env = {**os.environ, 'ATLAS_E2E_ENABLE_OTEL':'1'}
    rc = subprocess.call(['python3','packages/atlasctl/src/atlasctl/commands/ops/obs/contracts/check_trace_coverage.py'], env=env, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    if rc == 0:
        print('expected trace coverage check to fail for missing spans', file=sys.stderr)
        return 1
    print('trace missing spans regression drill passed')
    return 0
if __name__ == '__main__': raise SystemExit(main())
