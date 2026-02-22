#!/usr/bin/env python3
from __future__ import annotations

import os
import shutil
import subprocess
import threading
import time
from pathlib import Path


def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for parent in cur.parents:
        if all((parent / marker).exists() for marker in ("makefiles", "packages", "configs", "ops")):
            return parent
    raise RuntimeError("unable to resolve repo root")


def main() -> int:
    root = _repo_root()
    out_dir = Path(os.environ.get("OUT_DIR", str(root / "artifacts/perf/results")))
    out_dir.mkdir(parents=True, exist_ok=True)
    ns = os.environ.get("ATLAS_E2E_NAMESPACE", "atlas-e2e")
    release = os.environ.get("ATLAS_E2E_RELEASE_NAME", "atlas-e2e")
    service = os.environ.get("ATLAS_E2E_SERVICE_NAME", f"{release}-bijux-atlas")

    def rollout_task() -> None:
        time.sleep(5)
        subprocess.run(["kubectl", "-n", ns, "rollout", "restart", f"deploy/{service}"], check=False, stdout=subprocess.DEVNULL)
        subprocess.run(
            ["kubectl", "-n", ns, "rollout", "status", f"deploy/{service}", "--timeout=240s"],
            check=False,
            stdout=subprocess.DEVNULL,
        )

    if shutil.which("kubectl") and subprocess.run(["kubectl", "-n", ns, "get", "deploy", service], check=False, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL).returncode == 0:
        t = threading.Thread(target=rollout_task, daemon=True)
        t.start()

    subprocess.run(
        [str(root / "bin/atlasctl"), "ops", "load", "--report", "text", "run", "--suite", "load-under-rollout.json", "--out", str(out_dir)],
        check=True,
        cwd=str(root),
    )
    print("load-under-rollout complete")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
