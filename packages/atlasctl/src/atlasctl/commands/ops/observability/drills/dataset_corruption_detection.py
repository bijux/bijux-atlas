from __future__ import annotations
import subprocess

def main() -> int:
    subprocess.check_call(['cargo','test','-p','bijux-atlas-server','cache_manager_tests::chaos_mode_random_byte_corruption_never_serves_results','--','--exact'])
    print('dataset corruption detection drill passed')
    return 0
if __name__ == '__main__': raise SystemExit(main())
