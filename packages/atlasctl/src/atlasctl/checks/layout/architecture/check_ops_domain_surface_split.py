#!/usr/bin/env python3
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[7]
OPS_CMDS = ROOT / "packages/atlasctl/src/atlasctl/commands/ops"
IGNORE = {
    "__pycache__",
    "_shared",
    "internal",
    "runtime_modules",
    "meta",
    "docker",
    "observability",
    "ops_lint",
    "lint",
}


def main() -> int:
    cfg = json.loads((ROOT / "configs/ops/ops-domain-surface.json").read_text(encoding="utf-8"))
    expected = set(cfg["public_domains"])
    transitional = set(cfg.get("transitional_public_domains", []))
    actual = {
        p.name
        for p in OPS_CMDS.iterdir()
        if p.is_dir() and p.name not in IGNORE and (p / "command.py").exists()
    }
    errs: list[str] = []
    if len(expected) > int(cfg["max_public_domains"]):
        errs.append(f"target public ops domain count {len(expected)} exceeds max {cfg['max_public_domains']}")
    allowed_actual = expected | transitional
    if not actual.issubset(allowed_actual):
        errs.append(
            f"public ops domains contain undeclared entries: {sorted(actual - allowed_actual)} "
            f"(expected={sorted(expected)}, transitional={sorted(transitional)})"
        )
    if not expected.issubset(actual):
        errs.append(f"missing expected public ops domains: {sorted(expected - actual)}")
    if errs:
        print("\n".join(errs))
        return 1
    print("ops domain surface split OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
