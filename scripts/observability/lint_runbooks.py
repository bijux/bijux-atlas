#!/usr/bin/env python3
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
RUNBOOKS = sorted((ROOT / "docs/runbooks").glob("*.md"))
SERVER_SRC = (ROOT / "crates/bijux-atlas-server/src").read_text() if False else None

if not RUNBOOKS:
    print("no runbooks found", file=sys.stderr)
    sys.exit(1)

# Build target set from make metadata.
make_proc = subprocess.run(
    ["make", "-qp"],
    cwd=ROOT,
    text=True,
    stdout=subprocess.PIPE,
    stderr=subprocess.STDOUT,
    check=False,
)
make_db = make_proc.stdout
targets = set(re.findall(r"^([A-Za-z0-9_.%/+\-]+):", make_db, flags=re.MULTILINE))

server_paths = subprocess.check_output(
    ["rg", "-n", '"/(v1|metrics|healthz|readyz|debug)[^"\\s]*"', "crates/bijux-atlas-server/src", "-S"],
    cwd=ROOT,
    text=True,
)

errors = []
for rb in RUNBOOKS:
    text = rb.read_text()

    # Validate make targets referenced in backticks.
    for target in re.findall(r"`make\s+([a-zA-Z0-9_\-]+)`", text):
        if target not in targets:
            errors.append(f"{rb}: unknown make target `{target}`")

    # Validate endpoint references exist in server routes/source.
    for ep in re.findall(r"(/(?:v1|metrics|healthz|readyz|debug)[a-zA-Z0-9_\-/{}:?=&.]*)", text):
        ep_prefix = ep.split("?")[0]
        if ep_prefix not in server_paths:
            errors.append(f"{rb}: endpoint not found in server source `{ep_prefix}`")

if errors:
    print("runbook lint failed:", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    sys.exit(1)

print("runbook lint passed")
