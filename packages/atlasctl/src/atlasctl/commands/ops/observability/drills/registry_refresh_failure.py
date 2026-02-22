from __future__ import annotations
import subprocess

def main() -> int:
    subprocess.check_call(['./bin/atlasctl','run','./packages/atlasctl/src/atlasctl/commands/ops/k8s/tests/checks/datasets/test_readiness_catalog_refresh.py'])
    print('registry refresh failure drill passed')
    return 0
if __name__ == '__main__': raise SystemExit(main())
