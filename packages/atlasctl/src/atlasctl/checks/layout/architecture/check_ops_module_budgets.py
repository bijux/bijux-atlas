#!/usr/bin/env python3
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[7]
OPS_ROOT = ROOT / "packages/atlasctl/src/atlasctl/commands/ops"


def main() -> int:
    cfg = json.loads((ROOT / "configs/ops/ops-module-budgets.json").read_text(encoding="utf-8"))
    py_files = [p for p in OPS_ROOT.rglob("*.py") if "__pycache__" not in p.parts]
    max_depth = 0
    max_bytes = 0
    max_bytes_path = ""
    for p in py_files:
        rel = p.relative_to(OPS_ROOT)
        max_depth = max(max_depth, len(rel.parts))
        size = p.stat().st_size
        if size > max_bytes:
            max_bytes = size
            max_bytes_path = p.relative_to(ROOT).as_posix()
    errs: list[str] = []
    if len(py_files) > int(cfg["commands_ops_max_py_files"]):
        errs.append(f"commands/ops py file count {len(py_files)} exceeds budget")
    if max_depth > int(cfg["commands_ops_max_depth"]):
        errs.append(f"commands/ops max depth {max_depth} exceeds budget")
    if max_bytes > int(cfg["commands_ops_max_bytes_per_file"]):
        errs.append(f"largest commands/ops module exceeds budget: {max_bytes_path} ({max_bytes} bytes)")
    if errs:
        print("\n".join(errs))
        return 1
    print("ops module budgets OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
