#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[7]
OPS_DIR = ROOT / "packages/atlasctl/src/atlasctl/commands/ops"
FORBIDDEN_FLOATING = {
    "ops_k8s.py": "Move to commands/ops/k8s/runtime_bridge.py or another area module.",
}


def main() -> int:
    errs: list[str] = []
    for name, hint in FORBIDDEN_FLOATING.items():
        path = OPS_DIR / name
        if path.exists():
            errs.append(f"{path.relative_to(ROOT).as_posix()}: forbidden floating ops module. {hint}")
    if errs:
        print("\n".join(errs))
        return 1
    print("no forbidden floating ops modules")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
