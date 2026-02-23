from __future__ import annotations

K8S_WRAPPER_ACTIONS: dict[str, list[str]] = {
    'render': ['helm', 'template', 'atlas', 'ops/chart'],
    'install': ['./bin/atlasctl', 'ops', 'deploy', '--report', 'text', 'apply'],
    'uninstall': ['./bin/atlasctl', 'ops', 'deploy', '--report', 'text', 'rollback'],
}

STACK_WRAPPER_ACTIONS: dict[str, list[str]] = {
    'up': ['./bin/atlasctl', 'ops', 'stack', '--report', 'text', 'up'],
    'down': ['./bin/atlasctl', 'ops', 'stack', '--report', 'text', 'down'],
}

OBS_WRAPPER_ACTIONS: dict[str, list[str]] = {
    'up': ['./bin/atlasctl', 'ops', 'obs', '--report', 'text', 'up'],
    'verify': ['./bin/atlasctl', 'ops', 'obs', '--report', 'text', 'verify'],
    'down': ['./bin/atlasctl', 'ops', 'obs', '--report', 'text', 'validate'],
}

LOAD_WRAPPER_ACTIONS: dict[str, list[str]] = {
    'smoke': ['make', 'ops-load-smoke'],
    'suite': ['./bin/atlasctl', 'ops', 'load', '--report', 'text', 'run'],
    'baseline-compare': ['python3', 'packages/atlasctl/src/atlasctl/load/baseline/compare_runs.py'],
    'baseline-update': ['python3', 'packages/atlasctl/src/atlasctl/load/baseline/update_baseline.py'],
}

E2E_WRAPPER_ACTIONS: dict[str, list[str]] = {
    'smoke': ['./bin/atlasctl', 'ops', 'e2e', '--report', 'text', 'run', '--suite', 'smoke'],
    'realdata': ['./bin/atlasctl', 'ops', 'e2e', '--report', 'text', 'run', '--suite', 'realdata'],
}

DATASETS_WRAPPER_ACTIONS: dict[str, list[str]] = {
    'verify': ['./bin/atlasctl', 'ops', 'datasets', '--report', 'text', 'verify'],
    'fetch': ['./bin/atlasctl', 'ops', 'warm', '--report', 'text', '--mode', 'warmup'],
    'pin': ['python3', 'packages/atlasctl/src/atlasctl/datasets/build_manifest_lock.py'],
}
