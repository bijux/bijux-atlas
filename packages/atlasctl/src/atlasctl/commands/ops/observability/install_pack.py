from __future__ import annotations

import json
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


def _ensure_offline_images() -> int:
    cfg = json.loads(Path('configs/ops/observability-pack.json').read_text(encoding='utf-8'))
    for spec in cfg.get('images', {}).values():
        ref = spec['ref']
        rc = subprocess.call(['docker', 'image', 'inspect', ref], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        if rc != 0:
            print(f'offline mode missing image locally: {ref}', file=sys.stderr)
            return 1
    print('offline image precheck passed')
    return 0


def _kind_load_offline_images() -> int:
    cluster_name = os.environ.get('ATLAS_E2E_CLUSTER_NAME', 'bijux-atlas-e2e')
    cfg = json.loads(Path('configs/ops/observability-pack.json').read_text(encoding='utf-8'))
    for spec in cfg.get('images', {}).values():
        if subprocess.call(['kind', 'load', 'docker-image', spec['ref'], '--name', cluster_name]) != 0:
            return 1
    return 0


def main() -> int:
    profile = os.environ.get('ATLAS_OBS_PROFILE', 'kind')
    if len(sys.argv) > 2 and sys.argv[1] == '--profile':
        profile = sys.argv[2]
    obs_ns = os.environ.get('ATLAS_OBS_NAMESPACE', 'atlas-observability')
    storage_mode = os.environ.get('ATLAS_OBS_STORAGE_MODE', 'ephemeral')
    offline_mode = os.environ.get('ATLAS_OBS_OFFLINE', '0')
    root = Path.cwd()
    if profile == 'local-compose':
        cmd = _compose_cmd()
        if not cmd:
            print('local-compose profile requires docker compose or docker-compose', file=sys.stderr)
            return 1
        if offline_mode == '1':
            if _ensure_offline_images() != 0:
                return 1
            return subprocess.call([*cmd, '-f', str(root/'ops/obs/pack/compose/docker-compose.yml'), 'up', '-d', '--pull', 'never'])
        return subprocess.call([*cmd, '-f', str(root/'ops/obs/pack/compose/docker-compose.yml'), 'up', '-d'])
    if profile in {'kind', 'cluster'}:
        def k(*args: str) -> int:
            return subprocess.call(['kubectl', *args])
        if k('apply', '-f', str(root/'ops/obs/pack/k8s/namespace.yaml')) != 0: return 1
        if k('apply', '-f', str(root/'ops/obs/pack/k8s/rbac.yaml')) != 0: return 1
        if offline_mode == '1':
            if _ensure_offline_images() != 0: return 1
            if _kind_load_offline_images() != 0: return 1
        if k('apply', '-f', str(root/'ops/obs/pack/k8s/prometheus-config.yaml')) != 0: return 1
        if storage_mode == 'persistent':
            if k('apply', '-f', str(root/'ops/obs/pack/k8s/prometheus-pvc.yaml')) != 0: return 1
            if k('apply', '-f', str(root/'ops/obs/pack/k8s/grafana-pvc.yaml')) != 0: return 1
        for f in ('prometheus.yaml','grafana-config.yaml','grafana.yaml'):
            if k('apply','-f', str(root/'ops/obs/pack/k8s'/f)) != 0: return 1
        cfg_cmd = "kubectl -n {ns} create configmap atlas-observability-otel-config --from-file=config.yaml={cfg} --dry-run=client -o yaml | kubectl apply -f -".format(ns=obs_ns, cfg=str(root/'ops/obs/pack/otel/config.yaml'))
        if subprocess.call(['bash','-lc', cfg_cmd]) != 0: return 1
        if k('apply','-f', str(root/'ops/obs/pack/k8s/otel.yaml')) != 0: return 1
        api = subprocess.check_output(['kubectl','api-resources'], text=True)
        if 'prometheusrules' in api:
            if subprocess.call(['kubectl','-n',obs_ns,'apply','-f', str(root/'ops/obs/alerts/atlas-alert-rules.yaml')]) != 0: return 1
            if subprocess.call(['kubectl','-n',obs_ns,'apply','-f', str(root/'ops/obs/alerts/slo-burn-rules.yaml')]) != 0: return 1
        else:
            print('PrometheusRule CRD not present; continuing without rule install')
        if profile == 'cluster' and 'servicemonitors' not in api:
            print('cluster profile requested but ServiceMonitor CRD missing', file=sys.stderr)
            return 1
        if storage_mode == 'persistent':
            subprocess.call(['kubectl','-n',obs_ns,'patch','deploy','atlas-observability-prometheus','--type=json','-p','[{"op":"replace","path":"/spec/template/spec/volumes/1","value":{"name":"data","persistentVolumeClaim":{"claimName":"atlas-observability-prometheus-data"}}}]'])
            subprocess.call(['kubectl','-n',obs_ns,'patch','deploy','atlas-observability-grafana','--type=json','-p','[{"op":"add","path":"/spec/template/spec/volumes/-","value":{"name":"grafana-data","persistentVolumeClaim":{"claimName":"atlas-observability-grafana-data"}}},{"op":"add","path":"/spec/template/spec/containers/0/volumeMounts/-","value":{"name":"grafana-data","mountPath":"/var/lib/grafana"}}]'])
        subprocess.call(['bash', str(root/'ops/stack/minio/bootstrap.sh')], env={**os.environ, 'NS': os.environ.get('ATLAS_STACK_NAMESPACE', 'atlas-e2e')})
        return subprocess.call(['python3', 'packages/atlasctl/src/atlasctl/commands/ops/stack/wait_ready.py', os.environ.get('ATLAS_STACK_NAMESPACE', 'atlas-e2e'), os.environ.get('ATLAS_E2E_TIMEOUT', '180s')])
    print(f'unknown profile: {profile} (expected: local-compose|kind|cluster)', file=sys.stderr)
    return 1


if __name__ == '__main__':
    raise SystemExit(main())
