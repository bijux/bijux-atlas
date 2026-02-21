#!/usr/bin/env python3
from __future__ import annotations

import json
import os
import shutil
import subprocess
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[6]

TOOLS = ["python3", "cargo", "docker", "kind", "kubectl", "helm", "k6", "jq", "yq"]
KEY_ENV = ["RUN_ID", "ISO_ROOT", "CARGO_TARGET_DIR", "CARGO_HOME", "TMPDIR", "ATLAS_BASE_URL", "ATLAS_NS"]


def tool_version(tool: str) -> str:
    exe = shutil.which(tool)
    if not exe:
        return "missing"
    cmd = [tool, "--version"]
    if tool == "kubectl":
        cmd = [tool, "version", "--client", "--short"]
    if tool == "helm":
        cmd = [tool, "version", "--short"]
    try:
        out = subprocess.check_output(cmd, stderr=subprocess.STDOUT, text=True).strip()
        return out.splitlines()[0] if out else "ok"
    except Exception as exc:  # noqa: BLE001
        return f"error: {exc}"


def main() -> int:
    run_id = os.environ.get("RUN_ID", "doctor")
    out_dir = ROOT / "ops" / "_evidence" / "make" / run_id
    out_dir.mkdir(parents=True, exist_ok=True)
    out = out_dir / "doctor.json"

    payload = {
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "run_id": run_id,
        "tools": {tool: tool_version(tool) for tool in TOOLS},
        "paths": {
            "workspace": str(ROOT),
            "ops_evidence_make": str(ROOT / "ops" / "_evidence" / "make"),
            "artifacts_isolate": str(ROOT / "artifacts" / "isolate"),
        },
        "env": {k: os.environ.get(k, "") for k in KEY_ENV},
    }

    out.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(out.relative_to(ROOT))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
