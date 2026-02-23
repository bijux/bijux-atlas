from __future__ import annotations
import subprocess, sys
from atlasctl.core.runtime.repo_root import find_repo_root
from pathlib import Path

def main() -> int:
    root = find_repo_root()
    out = root / 'artifacts/observability/drills/metrics-cardinality.prom'
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(
        'bijux_http_requests_total{subsystem="atlas",route="/v1/genes",status="200",query_type="list",dataset="d1",version="v1",request_id="r1"} 1\n',
        encoding='utf-8',
    )
    rc = subprocess.call(['python3','packages/atlasctl/src/atlasctl/commands/ops/observability/check_metric_cardinality.py', str(out)], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    if rc == 0:
        print('expected metric cardinality check to fail', file=sys.stderr); return 1
    print('metric cardinality blowup attempt drill passed')
    return 0

if __name__ == '__main__': raise SystemExit(main())
