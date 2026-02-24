from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path


def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for parent in (cur, *cur.parents):
        if (parent / ".git").exists() and (parent / "makefiles").is_dir() and (parent / "configs").is_dir():
            return parent
    raise RuntimeError("unable to resolve repository root")


def _require_cmd(name: str) -> None:
    if subprocess.run(["bash", "-lc", f"command -v {name} >/dev/null 2>&1"]).returncode != 0:
        raise RuntimeError(f"required command not found: {name}")


def _ci_no_prompt_policy() -> None:
    # Atlasctl command is non-interactive by default; preserve the shell helper's intent.
    if os.environ.get("CI", "0") in {"1", "true", "TRUE"}:
        os.environ.setdefault("ATLASCTL_NONINTERACTIVE", "1")


def main() -> int:
    root = _repo_root()
    _ci_no_prompt_policy()
    try:
        _require_cmd("kind")
        _require_cmd("kubectl")
    except RuntimeError as exc:
        print(str(exc), file=sys.stderr)
        return 1

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
