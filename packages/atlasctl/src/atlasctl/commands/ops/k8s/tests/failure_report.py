#!/usr/bin/env python3
from __future__ import annotations

import os
import shutil
import subprocess
import sys
import tarfile
import time
from pathlib import Path


def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for parent in cur.parents:
        if all((parent / marker).exists() for marker in ("makefiles", "packages", "configs", "ops")):
            return parent
    raise RuntimeError("unable to resolve repo root")


def _run(cmd: list[str], cwd: Path, out_path: Path | None = None) -> None:
    proc = subprocess.run(cmd, cwd=str(cwd), text=True, capture_output=True, check=False)
    if out_path is not None:
        out_path.parent.mkdir(parents=True, exist_ok=True)
        out_path.write_text(proc.stdout or "", encoding="utf-8")


def collect_failure_report(repo_root: Path, namespace: str, release: str) -> Path:
    artifacts_root = Path(os.environ.get("ARTIFACTS_ROOT", str(repo_root / "artifacts")))
    ts = time.strftime("%Y%m%d-%H%M%S")
    out_dir = artifacts_root / "k8s-failures" / ts
    out_dir.mkdir(parents=True, exist_ok=True)

    # Preserve old helper behavior as closely as possible.
    _run(["kubectl", "get", "pods", "-A", "-o", "wide"], repo_root, out_dir / "pods.txt")
    _run(["kubectl", "-n", namespace, "get", "all", "-o", "wide"], repo_root, out_dir / f"all-{namespace}.txt")
    _run(["kubectl", "get", "events", "-A", "--sort-by=.lastTimestamp"], repo_root, out_dir / "events.txt")
    _run(["kubectl", "-n", namespace, "get", "configmap", release, "-o", "yaml"], repo_root, out_dir / "configmap.yaml")
    _run(["kubectl", "-n", namespace, "describe", "pods"], repo_root, out_dir / "describe-pods.txt")
    _run(["kubectl", "-n", namespace, "describe", "svc"], repo_root, out_dir / "describe-services.txt")
    _run(["kubectl", "-n", namespace, "describe", "deploy"], repo_root, out_dir / "describe-deployments.txt")
    _run(["kubectl", "-n", namespace, "describe", "hpa"], repo_root, out_dir / "describe-hpa.txt")
    _run(["kubectl", "-n", namespace, "get", "events", "--sort-by=.lastTimestamp"], repo_root, out_dir / "events.describe.txt")

    top_probe = subprocess.run(["kubectl", "top", "pods", "-n", namespace], cwd=str(repo_root), text=True, capture_output=True, check=False)
    if top_probe.returncode == 0:
        (out_dir / "top-pods.txt").write_text(top_probe.stdout or "", encoding="utf-8")

    tar_path = artifacts_root / f"k8s-failure-bundle-{ts}.tar.gz"
    try:
        with tarfile.open(tar_path, "w:gz") as tf:
            for child in sorted(out_dir.iterdir()):
                tf.add(child, arcname=child.name)
    except Exception:
        pass
    return out_dir


def main() -> int:
    root = _repo_root()
    ns = sys.argv[1] if len(sys.argv) > 1 else os.environ.get("ATLAS_E2E_NAMESPACE", f"atlas-e2e-{os.environ.get('USER', 'local')}")
    release = sys.argv[2] if len(sys.argv) > 2 else os.environ.get("ATLAS_E2E_RELEASE_NAME", "atlas-e2e")
    out = collect_failure_report(root, ns, release)
    print(f"k8s test report: {out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
