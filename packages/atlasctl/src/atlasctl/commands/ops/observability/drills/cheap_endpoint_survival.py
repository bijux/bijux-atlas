from __future__ import annotations
import os, subprocess, sys

def main() -> int:
    root = 'packages/atlasctl/src/atlasctl/commands/ops/observability/drills/overload_admission_control.py'
    base = os.environ.get('ATLAS_BASE_URL', 'http://127.0.0.1:18080')
    subprocess.check_call(['python3', root])
    subprocess.check_call(['curl','-fsS',f'{base}/v1/version'], stdout=subprocess.DEVNULL)
    out = subprocess.check_output(['curl','-fsS',f'{base}/metrics'], text=True)
    if 'bijux_cheap_queries_served_while_overloaded_total' not in out:
        print('missing cheap endpoint survival metric', file=sys.stderr); return 1
    print('cheap endpoint survival drill passed')
    return 0

if __name__ == '__main__': raise SystemExit(main())
