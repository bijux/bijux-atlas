from __future__ import annotations

import os
import subprocess
import sys
import time


def _default_namespace() -> str:
    return os.environ.get('ATLAS_E2E_NAMESPACE') or os.environ.get('ATLAS_NS') or 'atlas-e2e'


def _default_release() -> str:
    return os.environ.get('ATLAS_E2E_RELEASE_NAME') or 'atlas-e2e'


def main() -> int:
    ns = _default_namespace()
    release = _default_release()
    subprocess.call(['helm', '-n', ns, 'uninstall', release], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    subprocess.call(['kubectl', 'delete', 'ns', ns, '--ignore-not-found'], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    for _ in range(60):
        if subprocess.call(['kubectl', 'get', 'ns', ns], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL) != 0:
            break
        time.sleep(2)
    if subprocess.call(['kubectl', 'get', 'ns', ns], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL) == 0:
        print(f'namespace still exists after uninstall: {ns}', file=sys.stderr)
        return 1
    print(f'clean uninstall complete: ns={ns} release={release}')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
