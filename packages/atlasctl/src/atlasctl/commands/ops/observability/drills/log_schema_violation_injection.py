from __future__ import annotations
import subprocess
from atlasctl.core.runtime.repo_root import find_repo_root
import sys
from pathlib import Path

def main() -> int:
    root = find_repo_root()
    out = root / 'artifacts/observability/drills/log-schema-violation.jsonl'
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text('{"event":"request_end","request_id":123,"dataset":null}\n', encoding='utf-8')
    rc = subprocess.call(['python3','packages/atlasctl/src/atlasctl/commands/ops/observability/validate_logs_schema.py','--file',str(out)], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    if rc == 0:
        print('expected log schema validator to fail', file=sys.stderr); return 1
    print('log schema violation injection drill passed')
    return 0

if __name__ == '__main__': raise SystemExit(main())
