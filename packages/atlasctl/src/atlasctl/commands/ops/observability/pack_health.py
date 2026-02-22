from __future__ import annotations

import os
import subprocess


def main() -> int:
    profile = os.environ.get('ATLAS_OBS_PROFILE', 'kind')
    base = os.environ.get('ATLAS_BASE_URL', 'http://127.0.0.1:18080')
    subprocess.check_call(['python3', 'packages/atlasctl/src/atlasctl/commands/ops/observability/verify_pack.py', '--profile', profile])
    subprocess.check_call(['curl', '-fsS', f'{base}/readyz'], stdout=subprocess.DEVNULL)
    print(f'pack health ok (profile={profile})')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
