#!/usr/bin/env python3
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[7]
CFG = ROOT / "configs/ops/vendor-allow.json"


def main() -> int:
    cfg = json.loads(CFG.read_text(encoding="utf-8"))
    max_files = int(cfg.get("max_files", 0))
    allowed = {str(x) for x in cfg.get("paths", [])}
    vendor_files = sorted(
        p.relative_to(ROOT).as_posix()
        for p in (ROOT / "ops/vendor").rglob("*")
        if p.is_file()
    )
    errs: list[str] = []
    if len(vendor_files) > max_files:
        errs.append(f"ops/vendor file count {len(vendor_files)} exceeds max_files={max_files}")
    for rel in vendor_files:
        if rel not in allowed:
            errs.append(f"ops/vendor file not explicitly allowed: {rel}")
    if errs:
        print("\n".join(errs))
        return 1
    print("ops/vendor budget and owner allowlist OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
