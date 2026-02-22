#!/usr/bin/env python3
from __future__ import annotations

import subprocess
import sys
from pathlib import Path


def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for parent in cur.parents:
        if all((parent / marker).exists() for marker in ("makefiles", "packages", "configs", "ops")):
            return parent
    raise RuntimeError("unable to resolve repo root")


def _run(cmd: list[str], root: Path, check: bool = True) -> subprocess.CompletedProcess[str]:
    return subprocess.run(cmd, cwd=str(root), check=check, text=True, capture_output=True)


def main() -> int:
    root = _repo_root()
    cmds = [
        ["kubectl", "wait", "--for=condition=Ready", "nodes", "--all", "--timeout=120s"],
        ["kubectl", "-n", "kube-system", "rollout", "status", "deploy/coredns", "--timeout=120s"],
        ["kubectl", "get", "storageclass"],
    ]
    for cmd in cmds:
        proc = _run(cmd, root, check=False)
        if proc.returncode != 0:
            sys.stderr.write(proc.stderr or proc.stdout)
            return proc.returncode
    proc = _run(
        [
            "kubectl",
            "get",
            "nodes",
            "-o",
            'jsonpath={.items[0].status.conditions[?(@.type=="Ready")].status}',
        ],
        root,
        check=False,
    )
    if proc.returncode != 0:
        sys.stderr.write(proc.stderr or proc.stdout)
        return proc.returncode
    if "True" not in (proc.stdout or ""):
        print("node not ready", file=sys.stderr)
        return 1
    print("cluster sanity passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
