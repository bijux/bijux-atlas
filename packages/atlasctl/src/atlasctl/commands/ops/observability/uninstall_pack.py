from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path


def _compose_cmd() -> list[str] | None:
    if subprocess.call(['docker', 'compose', 'version'], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL) == 0:
        return ['docker', 'compose']
    if subprocess.call(['docker-compose', '--version'], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL) == 0:
        return ['docker-compose']
    return None


def main() -> int:
    profile = os.environ.get('ATLAS_OBS_PROFILE', 'kind')
    if len(sys.argv) > 2 and sys.argv[1] == '--profile':
        profile = sys.argv[2]
    obs_ns = os.environ.get('ATLAS_OBS_NAMESPACE', 'atlas-observability')
    root = Path.cwd()
    if profile == 'local-compose':
        cmd = _compose_cmd()
        if not cmd:
            print('local-compose profile requires docker compose or docker-compose', file=sys.stderr)
            return 1
        return subprocess.call([*cmd, '-f', str(root/'ops/obs/pack/compose/docker-compose.yml'), 'down', '-v'])
    if profile in {'kind','cluster'}:
        calls = [
            ['kubectl','-n',obs_ns,'delete','-f', str(root/'ops/obs/alerts/slo-burn-rules.yaml'), '--ignore-not-found'],
            ['kubectl','-n',obs_ns,'delete','-f', str(root/'ops/obs/alerts/atlas-alert-rules.yaml'), '--ignore-not-found'],
            ['kubectl','delete','-f', str(root/'ops/obs/pack/k8s/otel.yaml'), '--ignore-not-found'],
            ['kubectl','-n',obs_ns,'delete','configmap','atlas-observability-otel-config', '--ignore-not-found'],
            ['kubectl','delete','-f', str(root/'ops/obs/pack/k8s/grafana.yaml'), '--ignore-not-found'],
            ['kubectl','delete','-f', str(root/'ops/obs/pack/k8s/grafana-config.yaml'), '--ignore-not-found'],
            ['kubectl','delete','-f', str(root/'ops/obs/pack/k8s/prometheus.yaml'), '--ignore-not-found'],
            ['kubectl','delete','-f', str(root/'ops/obs/pack/k8s/prometheus-config.yaml'), '--ignore-not-found'],
            ['kubectl','delete','-f', str(root/'ops/obs/pack/k8s/prometheus-pvc.yaml'), '--ignore-not-found'],
            ['kubectl','delete','-f', str(root/'ops/obs/pack/k8s/grafana-pvc.yaml'), '--ignore-not-found'],
            ['kubectl','delete','-f', str(root/'ops/obs/pack/k8s/rbac.yaml'), '--ignore-not-found'],
            ['kubectl','delete','-f', str(root/'ops/obs/pack/k8s/namespace.yaml'), '--ignore-not-found'],
        ]
        for cmd in calls:
            subprocess.call(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        print(f'observability pack uninstalled (profile={profile})')
        return 0
    print(f'unknown profile: {profile} (expected: local-compose|kind|cluster)', file=sys.stderr)
    return 1


if __name__ == '__main__':
    raise SystemExit(main())
