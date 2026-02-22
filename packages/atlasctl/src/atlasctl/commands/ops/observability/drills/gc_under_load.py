from __future__ import annotations
import os, subprocess, time

def main() -> int:
    base = os.environ.get('ATLAS_BASE_URL','http://127.0.0.1:18080')
    bg = subprocess.Popen(['bash','-lc', f'for _ in $(seq 1 10); do curl -fsS "{base}/v1/version" >/dev/null || true; sleep 1; done'])
    try:
        subprocess.check_call(['make','ops-gc-smoke'])
    finally:
        try:
            bg.wait(timeout=15)
        except Exception:
            bg.kill()
    print('gc under load drill passed')
    return 0
if __name__ == '__main__': raise SystemExit(main())
