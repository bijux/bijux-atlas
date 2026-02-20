#!/usr/bin/env bash
# Purpose: assert repository root shape against a categorized JSON whitelist.
# Inputs: scripts/areas/layout/root_whitelist.json and root filesystem entries.
# Outputs: non-zero exit on missing required or unexpected root entries.
set -euo pipefail

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
ALLOW_JSON="$ROOT/scripts/areas/layout/root_whitelist.json"

python3 - "$ROOT" "$ALLOW_JSON" <<'PY'
from __future__ import annotations
import json
from pathlib import Path
import subprocess
import sys

root = Path(sys.argv[1])
cfg = json.loads(Path(sys.argv[2]).read_text())
required = set(cfg.get("required", []))
allowed = set(cfg.get("allowed", []))
compat = set(cfg.get("compat_shims", []))
local_noise = set(cfg.get("local_noise", []))
allow = required | allowed | compat | local_noise
entries = set()
noise_seen = set()
for p in root.iterdir():
    if p.name == ".git":
        continue
    # ignore unknown local git-ignored clutter
    ignored = subprocess.run(
        ["git", "check-ignore", "-q", p.name],
        cwd=root,
    ).returncode == 0
    if ignored and p.name not in allow:
        continue
    entries.add(p.name)
    if p.name in local_noise:
        noise_seen.add(p.name)
missing_required = sorted(required - entries)
unexpected = sorted(entries - allow)
if missing_required or unexpected:
    print("root shape check failed: unexpected root entries", file=sys.stderr)
    for name in missing_required:
        print(f"- missing required: {name}", file=sys.stderr)
    for name in unexpected:
        print(f"- unexpected: {name}", file=sys.stderr)
    raise SystemExit(1)
if noise_seen:
    print("root shape check warning: local-noise entries present (allowed locally)", file=sys.stderr)
    for name in sorted(noise_seen):
        print(f"- local-noise: {name}", file=sys.stderr)
print("root shape check passed")
PY
