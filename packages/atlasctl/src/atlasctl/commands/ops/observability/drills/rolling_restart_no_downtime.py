from __future__ import annotations
import subprocess

def main() -> int:
    subprocess.check_call(['./bin/atlasctl','run','./packages/atlasctl/src/atlasctl/commands/ops/k8s/tests/checks/rollout/test_rolling_restart_no_downtime.py'])
    print('rolling restart no downtime drill passed')
    return 0

if __name__ == '__main__': raise SystemExit(main())
