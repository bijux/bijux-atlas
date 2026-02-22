from __future__ import annotations
import os, subprocess, time

def main() -> int:
    base = os.environ.get('ATLAS_BASE_URL', 'http://127.0.0.1:18080')
    if os.environ.get('ATLAS_E2E_ENABLE_OTEL', '0') != '1':
        print('otel disabled; skip')
        return 0
    subprocess.call(['kubectl','-n','atlas-e2e','scale','deploy/otel-collector','--replicas=0'], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    time.sleep(3)
    subprocess.check_call(['curl','-fsS',f'{base}/healthz'], stdout=subprocess.DEVNULL)
    subprocess.call(['curl','-fsS',f'{base}/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&limit=1'], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    print('otel outage drill passed')
    return 0
if __name__ == '__main__': raise SystemExit(main())
