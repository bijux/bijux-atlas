from __future__ import annotations

import os
import subprocess
import time


def _rss_kib(ns: str, pod: str) -> int:
    out = subprocess.check_output(['kubectl', '-n', ns, 'exec', pod, '--', 'awk', '/VmRSS/ {print $2}', '/proc/1/status'], text=True).strip()
    return int(out or '0')


def main() -> int:
    base = os.environ.get('ATLAS_E2E_BASE_URL', 'http://127.0.0.1:18080')
    ns = os.environ.get('ATLAS_E2E_NAMESPACE', 'atlas-e2e')
    release = os.environ.get('ATLAS_E2E_RELEASE_NAME', 'atlas-e2e')
    duration = int(os.environ.get('ATLAS_E2E_SOAK_SECS', '600'))
    max_growth = int(os.environ.get('ATLAS_E2E_SOAK_MAX_GROWTH_KIB', '131072'))

    pod = subprocess.check_output([
        'kubectl', '-n', ns, 'get', 'pod', '-l', f'app.kubernetes.io/instance={release}',
        '-o', "jsonpath={.items[0].metadata.name}",
    ], text=True).strip()
    rss0 = _rss_kib(ns, pod)
    end = int(time.time()) + duration
    while int(time.time()) < end:
        subprocess.check_call(['curl', '-fsS', f'{base}/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&limit=10'], stdout=subprocess.DEVNULL)
        subprocess.call(['curl', '-fsS', f'{base}/v1/sequence/region?release=110&species=homo_sapiens&assembly=GRCh38&region=chr1:1-120'], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        time.sleep(1)
    rss1 = _rss_kib(ns, pod)
    growth = rss1 - rss0
    if growth > max_growth:
        print(f'memory growth too high: {growth}KiB', file=__import__('sys').stderr)
        return 1
    print(f'soak passed: rss_growth_kib={growth}')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
