#!/usr/bin/env python3
# Purpose: enforce shim minimality and deterministic behavior.
# Inputs: configs/layout/script-shim-expiries.json and shim files.
# Outputs: non-zero on non-minimal or unsafe shim shape.
from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
CFG = ROOT / "configs/layout/script-shim-expiries.json"


def main() -> int:
    payload = json.loads(CFG.read_text(encoding="utf-8"))
    errors: list[str] = []
    for row in payload.get("shims", []):
        if not isinstance(row, dict):
            continue
        rel = str(row.get("path", ""))
        if not rel:
            continue
        path = ROOT / rel
        if not path.exists():
            continue
        text = path.read_text(encoding="utf-8", errors="ignore")
        lines = [ln.strip() for ln in text.splitlines() if ln.strip()]
        if not lines or not lines[0].startswith("#!/usr/bin/env sh"):
            errors.append(f"{rel}: shim must use portable sh shebang")
        if "DEPRECATED:" not in text:
            errors.append(f"{rel}: missing DEPRECATED warning banner")
        if "docs/development/tooling/bijux-atlas-scripts.md" not in text:
            errors.append(f"{rel}: missing migration doc link")
        if "exec " not in text:
            errors.append(f"{rel}: missing exec passthrough")
        if any(tok in text for tok in ("tee ", "mktemp", "touch ", "cat > ", "printf > ", "echo > ")):
            errors.append(f"{rel}: shim must not write artifacts/files")
        if "set -x" in text or "uname" in text or "if [ \"$OSTYPE\"" in text:
            errors.append(f"{rel}: shim must be deterministic and OS-neutral")
        # minimal shim contract: comments + one echo + one exec
        non_comment = [ln for ln in lines if not ln.startswith("#")]
        if len(non_comment) > 2:
            errors.append(f"{rel}: shim must stay minimal (echo + exec only)")
    if errors:
        print("script shim minimality check failed:", file=sys.stderr)
        for err in errors:
            print(f"- {err}", file=sys.stderr)
        return 1
    print("script shim minimality check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
