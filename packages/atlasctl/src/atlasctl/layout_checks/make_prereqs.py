#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]
REQUIRED = ["python3", "cargo", "docker", "kind", "kubectl", "helm", "k6"]


def version(tool: str) -> str:
    cmd = [tool, "--version"]
    if tool == "kubectl":
        cmd = [tool, "version", "--client", "--short"]
    if tool == "helm":
        cmd = [tool, "version", "--short"]
    return subprocess.check_output(cmd, stderr=subprocess.STDOUT, text=True).strip().splitlines()[0]


def main() -> int:
    p = argparse.ArgumentParser()
    p.add_argument("--run-id", default=os.environ.get("RUN_ID", "prereqs"))
    args = p.parse_args()

    missing = [tool for tool in REQUIRED if shutil.which(tool) is None]
    status = "pass" if not missing else "fail"

    versions = {}
    for tool in REQUIRED:
        if tool in missing:
            versions[tool] = "missing"
            continue
        try:
            versions[tool] = version(tool)
        except Exception as exc:  # noqa: BLE001
            versions[tool] = f"error: {exc}"
            status = "fail"

    out_dir = ROOT / "ops" / "_evidence" / "make" / args.run_id
    out_dir.mkdir(parents=True, exist_ok=True)
    out = out_dir / "prereqs.json"
    payload = {
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "run_id": args.run_id,
        "status": status,
        "missing": missing,
        "versions": versions,
    }
    out.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(out.relative_to(ROOT))

    if status != "pass":
        for tool in missing:
            print(f"missing required tool: {tool}")
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
