#!/usr/bin/env python3
# Purpose: enforce ops run entrypoint wrapper contract and guard usage.
# Inputs: ops/run/*.sh wrappers.
# Outputs: non-zero exit on missing common bootstrap or forbidden direct network calls.
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[6]
RUN_DIR = ROOT / "ops" / "run"
NET_CMDS = ("curl", "kubectl", "helm")

errors = []
for script in sorted(RUN_DIR.glob("*.sh")):
    text = script.read_text(encoding="utf-8")
    if '. "$ROOT/ops/_lib/common.sh"' not in text:
        errors.append(f"{script}: missing common.sh import")
    if "ops_entrypoint_start " not in text:
        errors.append(f"{script}: missing ops_entrypoint_start call")
    has_version_guard = "ops_version_guard " in text
    if not has_version_guard:
        errors.append(f"{script}: missing ops_version_guard call")
    for cmd in NET_CMDS:
        if script.name == "prereqs.sh":
            continue
        if has_version_guard:
            continue
        if re.search(rf"(^|[^\w-]){re.escape(cmd)}\s", text):
            errors.append(f"{script}: direct network/tool call `{cmd}` not allowed in ops/run wrappers")

if errors:
    print("ops run entrypoint contract failed", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    sys.exit(1)

print("ops run entrypoint contract passed")
