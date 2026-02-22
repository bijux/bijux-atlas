from __future__ import annotations
import os, subprocess, time

def main() -> int:
    base = os.environ.get('ATLAS_BASE_URL', 'http://127.0.0.1:18080')
    subprocess.call(['kubectl','-n','atlas-e2e','scale','deploy/prometheus','--replicas=0'], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    time.sleep(3)
    subprocess.check_call(['curl','-fsS',f'{base}/healthz'], stdout=subprocess.DEVNULL)
    subprocess.check_call(['curl','-fsS',f'{base}/metrics'], stdout=subprocess.DEVNULL)
    print('prometheus outage drill passed')
    return 0
if __name__ == '__main__': raise SystemExit(main())
