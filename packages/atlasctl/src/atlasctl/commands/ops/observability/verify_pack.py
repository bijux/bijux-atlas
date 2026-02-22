from __future__ import annotations

import os
import subprocess
import sys
import tempfile
import time
from pathlib import Path


def _probe_http(url: str, label: str) -> int:
    for _ in range(30):
        if subprocess.call(['curl', '-sS', '-o', '/dev/null', url], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL) == 0:
            print(f'{label} reachable: {url}')
            return 0
        time.sleep(1)
    print(f'{label} unreachable: {url}', file=sys.stderr)
    return 1


def _ports_env() -> dict[str, str]:
    out = subprocess.check_output(['python3', 'packages/atlasctl/src/atlasctl/commands/ops/observability/pack_ports.py'], text=True)
    env = {}
    for line in out.splitlines():
        if '=' in line:
            k, v = line.split('=', 1)
            env[k] = v
    return env


def main() -> int:
    profile = os.environ.get('ATLAS_OBS_PROFILE', 'kind')
    if len(sys.argv) > 2 and sys.argv[1] == '--profile':
        profile = sys.argv[2]
    if profile == 'local-compose':
        env = _ports_env()
        if _probe_http(f"{env['ATLAS_PROM_URL']}/-/ready", 'prometheus') != 0:
            return 1
        if _probe_http(f"{env['ATLAS_GRAFANA_URL']}/api/health", 'grafana') != 0:
            return 1
        if _probe_http(f"{env['ATLAS_OTEL_HTTP_URL']}/", 'otel-collector') != 0:
            return 1
    elif profile in {'kind', 'cluster'}:
        ns = os.environ.get('ATLAS_OBS_NAMESPACE', 'atlas-observability')
        timeout = os.environ.get('OPS_WAIT_TIMEOUT', '180s')
        waits = [
            ['kubectl', '-n', ns, 'wait', '--for=condition=available', 'deploy/atlas-observability-prometheus', f'--timeout={timeout}'],
            ['kubectl', '-n', ns, 'wait', '--for=condition=available', 'deploy/atlas-observability-grafana', f'--timeout={timeout}'],
            ['kubectl', '-n', ns, 'wait', '--for=condition=available', 'deploy/atlas-observability-otel', f'--timeout={timeout}'],
        ]
        for cmd in waits:
            if subprocess.call(cmd) != 0:
                return 1
        with tempfile.NamedTemporaryFile('w+', delete=False) as pf:
            pid_file = pf.name
        cmd = f"kubectl -n {ns} port-forward svc/atlas-observability-prometheus 19090:9090 >/dev/null 2>&1 & echo $! > {pid_file}"
        subprocess.check_call(['bash', '-lc', cmd])
        pf_pid = Path(pid_file).read_text(encoding='utf-8').strip()
        Path(pid_file).unlink(missing_ok=True)
        try:
            ok = False
            for _ in range(30):
                probe_cmd = (
                    "curl -fsS http://127.0.0.1:19090/api/v1/targets | "
                    "python3 -c 'import json,sys; d=json.load(sys.stdin); "
                    "a=d.get(\"data\",{}).get(\"activeTargets\",[]); "
                    "sys.exit(0 if len(a)>0 else 1)'"
                )
                probe = subprocess.call(['bash', '-lc', probe_cmd], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
                if probe == 0:
                    print('prometheus targets up')
                    ok = True
                    break
                time.sleep(1)
            if not ok:
                return 1
        finally:
            subprocess.call(['bash', '-lc', f'kill {pf_pid} >/dev/null 2>&1 || true'])
    else:
        print(f'unknown profile: {profile} (expected: local-compose|kind|cluster)', file=sys.stderr)
        return 1
    print(f'observability pack verified (profile={profile})')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
