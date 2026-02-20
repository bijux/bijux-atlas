from __future__ import annotations

import subprocess
from datetime import datetime, timezone
from pathlib import Path


def make_run_id(prefix: str = "scripts") -> str:
    ts = datetime.now(timezone.utc).strftime("%Y%m%d-%H%M%S")
    sha = "unknown"
    try:
        root = Path(__file__).resolve().parents[4]
        out = subprocess.check_output(["git", "rev-parse", "--short", "HEAD"], cwd=root, text=True).strip()
        if out:
            sha = out
    except Exception:
        pass
    return f"{prefix}-{ts}-{sha}"
