#!/usr/bin/env bash
# Purpose: assert repository root shape against a JSON whitelist.
# Inputs: scripts/layout/allowed_root.json and root filesystem entries.
# Outputs: non-zero exit on unexpected root entries.
set -euo pipefail

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
ALLOW_JSON="$ROOT/scripts/layout/allowed_root.json"

python3 - "$ROOT" "$ALLOW_JSON" <<'PY'
from __future__ import annotations
import json
from pathlib import Path
import subprocess
import sys

root = Path(sys.argv[1])
allow = set(json.loads(Path(sys.argv[2]).read_text()).get("allowed", []))
entries = set()
for p in root.iterdir():
    if p.name == ".git":
        continue
    # ignore local, git-ignored clutter (e.g., .idea)
    ignored = subprocess.run(
        ["git", "check-ignore", "-q", p.name],
        cwd=root,
    ).returncode == 0
    if ignored:
        continue
    entries.add(p.name)
unexpected = sorted(entries - allow)
if unexpected:
    print("root shape check failed: unexpected root entries", file=sys.stderr)
    for name in unexpected:
        print(f"- {name}", file=sys.stderr)
    raise SystemExit(1)
print("root shape check passed")
PY
