from __future__ import annotations
import subprocess

def main() -> int:
    subprocess.check_call(['bash','ops/load/tests/test_cpu_throttle_noisy_neighbor.sh'])
    print('cpu throttle noisy neighbor drill passed')
    return 0
if __name__ == '__main__': raise SystemExit(main())
