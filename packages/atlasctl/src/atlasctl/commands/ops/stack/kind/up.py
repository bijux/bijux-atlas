from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path


def _repo_root() -> Path:
    return Path(__file__).resolve().parents[7]


def _source_common_and_run(script: str) -> None:
    root = _repo_root()
    subprocess.run(["bash", "-ceu", script], check=True, env={**os.environ, "ATLAS_REPO_ROOT": str(root)})


def main() -> int:
    root = _repo_root()
    # Preserve existing init/guard behavior from ops shell library while moving entrypoint ownership to atlasctl.
    _source_common_and_run(
        'ROOT="$ATLAS_REPO_ROOT"\n'
        'source "$ROOT/ops/_lib/common.sh"\n'
        'ops_init_run_id\n'
        'ops_ci_no_prompt_policy\n'
        'ops_version_guard kind kubectl\n'
    )

    cluster = os.environ.get("ATLAS_E2E_CLUSTER_NAME", "bijux-atlas-e2e")
    profile = os.environ.get("ATLAS_KIND_PROFILE", "normal")
    profile_to_cfg = {
        "small": root / "ops/stack/kind/cluster-small.yaml",
        "normal": root / "ops/stack/kind/cluster.yaml",
        "perf": root / "ops/stack/kind/cluster-perf.yaml",
    }
    cfg = profile_to_cfg.get(profile)
    if cfg is None:
        print(f"unknown ATLAS_KIND_PROFILE={profile} (small|normal|perf)", file=sys.stderr)
        return 2

    clusters = subprocess.run(["kind", "get", "clusters"], capture_output=True, text=True, check=True).stdout.splitlines()
    if cluster in clusters:
        print(f"kind cluster already exists: {cluster}")
        return 0
    if os.environ.get("OPS_DRY_RUN", "0") == "1":
        print(f"DRY-RUN kind create cluster --name {cluster} --config {cfg}")
        return 0
    subprocess.run(["kind", "create", "cluster", "--name", cluster, "--config", str(cfg)], check=True)
    print(f"kind cluster up: {cluster} profile={profile}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
