#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[6]
ROOT_MK = ROOT / "makefiles" / "root.mk"
DEV_MK = ROOT / "makefiles" / "dev.mk"

TARGET_RE = re.compile(r"^([A-Za-z0-9_./-]+):(?:\s|$)", re.M)

ROOT_CRITICAL = {"root", "root-local", "lane-cargo", "cargo/all", "ci/all"}


def main() -> int:
    root_text = ROOT_MK.read_text(encoding="utf-8")
    if not DEV_MK.exists():
        return 1, ["makefiles/dev.mk missing"]
    dev_text = DEV_MK.read_text(encoding="utf-8")
    dev_targets = {t for t in TARGET_RE.findall(dev_text) if not t.startswith(".")}

    errors: list[str] = []

    current_target = None
    for line in root_text.splitlines():
        m = re.match(r"^([A-Za-z0-9_./-]+):", line)
        if m:
            current_target = m.group(1)
            continue
        if current_target not in ROOT_CRITICAL:
            continue
        if current_target is None:
            continue
        if any(re.search(rf"\b{re.escape(dev)}\b", line) for dev in dev_targets):
            errors.append(f"{current_target} references dev wrapper target in recipe: {line.strip()}")

    if errors:
        print("root/dev wrapper boundary check failed", file=sys.stderr)
        for e in errors:
            print(f"- {e}", file=sys.stderr)
        return 1

    print("root/dev wrapper boundary check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
