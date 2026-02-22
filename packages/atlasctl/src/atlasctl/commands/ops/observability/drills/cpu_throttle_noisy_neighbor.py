from __future__ import annotations
import subprocess

def main() -> int:
    subprocess.check_call(['python3','packages/atlasctl/src/atlasctl/commands/ops/load/tests/test_cpu_throttle_noisy_neighbor.py'])
    print('cpu throttle noisy neighbor drill passed')
    return 0
if __name__ == '__main__': raise SystemExit(main())
