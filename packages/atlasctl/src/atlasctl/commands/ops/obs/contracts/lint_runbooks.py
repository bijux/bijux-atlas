#!/usr/bin/env python3
# Purpose: script interface entrypoint.
# Inputs: command-line args and repository files/env as documented by caller.
# Outputs: exit status and deterministic stdout/stderr or generated artifacts.
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


ROOT = _repo_root()
cmd = [sys.executable, "-m", "atlasctl.cli", "docs", "runbooks-contract-check", "--report", "text"]
raise SystemExit(subprocess.run(cmd, cwd=ROOT).returncode)
