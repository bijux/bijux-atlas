#!/usr/bin/env python3
# Purpose: script interface entrypoint.
# Inputs: command-line args and repository files/env as documented by caller.
# Outputs: exit status and deterministic stdout/stderr or generated artifacts.
from __future__ import annotations

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
cmd = [sys.executable, "-m", "atlasctl.cli", "docs", "runbooks-contract-check", "--report", "text"]
raise SystemExit(subprocess.run(cmd, cwd=ROOT).returncode)
