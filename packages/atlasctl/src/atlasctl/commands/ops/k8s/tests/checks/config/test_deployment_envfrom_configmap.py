#!/usr/bin/env python3
from __future__ import annotations

import os
import subprocess
import sys
import tempfile
from pathlib import Path


def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for parent in cur.parents:
        if all((parent / marker).exists() for marker in ("makefiles", "packages", "configs", "ops")):
            return parent
    raise RuntimeError("unable to resolve repo root")


def main() -> int:
    root = _repo_root()
    ns = os.environ.get("ATLAS_E2E_NAMESPACE", "atlas-e2e")
    release = os.environ.get("ATLAS_E2E_RELEASE_NAME", "atlas-e2e")
    values = os.environ.get("ATLAS_E2E_VALUES_FILE", str(root / "ops/k8s/values/local.yaml"))
    chart = root / "ops/k8s/charts/bijux-atlas"
    service_name = os.environ.get("ATLAS_E2E_SERVICE_NAME", f"{release}-bijux-atlas")
    with tempfile.NamedTemporaryFile("w+", delete=False) as tmp:
        tmp_path = Path(tmp.name)
    try:
        proc = subprocess.run(
            ["helm", "template", release, str(chart), "-n", ns, "-f", values],
            cwd=str(root),
            check=False,
            text=True,
            capture_output=True,
        )
        if proc.returncode != 0:
            sys.stderr.write(proc.stderr or proc.stdout)
            return proc.returncode
        rendered = proc.stdout
        tmp_path.write_text(rendered, encoding="utf-8")
        required = ["kind: Deployment", "envFrom:", "configMapRef:", f"name: {service_name}-config"]
        for needle in required:
            if needle not in rendered:
                print(f"deployment envFrom configmap contract failed: missing `{needle}`", file=sys.stderr)
                return 1
        print("deployment envFrom configmap contract passed")
        return 0
    finally:
        try:
            tmp_path.unlink()
        except FileNotFoundError:
            pass


if __name__ == "__main__":
    raise SystemExit(main())
