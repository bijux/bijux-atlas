from __future__ import annotations
import subprocess
from atlasctl.core.runtime.repo_root import find_repo_root
from pathlib import Path


def main() -> int:
    root = find_repo_root()
    for rb in ('store-outage.md','traffic-spike.md','rollback-playbook.md','pod-churn.md','dataset-corruption.md'):
        p = root / 'docs/operations/runbooks' / rb
        if not p.is_file() or p.stat().st_size == 0:
            raise SystemExit(f'missing runbook: {rb}')
    for drill in ('store-outage-under-load','overload-admission-control','prom-outage','otel-outage'):
        subprocess.check_call(['python3','packages/atlasctl/src/atlasctl/commands/ops/observability/drills/run_drill.py', drill])
    print('runbook proof top5 drill passed')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
