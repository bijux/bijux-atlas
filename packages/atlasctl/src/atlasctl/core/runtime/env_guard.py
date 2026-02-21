from __future__ import annotations

import os
from pathlib import Path


def guard_no_network_mode(no_network: bool) -> None:
    if no_network:
        os.environ['BIJUX_SCRIPTS_NO_NETWORK'] = '1'


def guard_kube_context_allowed(allowed_prefixes: tuple[str, ...] = ('kind-', 'atlas-')) -> tuple[bool, str]:
    try:
        import subprocess

        out = subprocess.check_output(['kubectl', 'config', 'current-context'], text=True).strip()
    except Exception:
        return True, 'kubectl unavailable; skipped'
    if out.startswith(allowed_prefixes):
        return True, out
    return False, out


def guard_kind_cluster_expected(cluster_name: str | None = None) -> tuple[bool, str]:
    expected = cluster_name or os.environ.get('ATLAS_E2E_CLUSTER_NAME', '')
    if not expected:
        return True, 'no expected cluster configured'
    try:
        import subprocess

        out = subprocess.check_output(['kind', 'get', 'clusters'], text=True)
    except Exception:
        return True, 'kind unavailable; skipped'
    clusters = {line.strip() for line in out.splitlines() if line.strip()}
    return (expected in clusters), expected


def ensure_within_repo(repo_root: Path, path: Path) -> bool:
    resolved = path.resolve()
    return repo_root.resolve() == resolved or repo_root.resolve() in resolved.parents
