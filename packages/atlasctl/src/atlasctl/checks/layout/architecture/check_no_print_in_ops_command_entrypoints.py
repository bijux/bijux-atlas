#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[7]
OPS_CMDS = ROOT / "packages/atlasctl/src/atlasctl/commands/ops"
ALLOW = {
    "packages/atlasctl/src/atlasctl/commands/ops/command.py",  # top-level parser/help facade (transitional)
    "packages/atlasctl/src/atlasctl/commands/ops/orchestrate/command.py",  # transitional orchestrate facade
}


def main() -> int:
    errs: list[str] = []
    for path in sorted(OPS_CMDS.rglob("command.py")):
        rel = path.relative_to(ROOT).as_posix()
        if rel in ALLOW:
            continue
        text = path.read_text(encoding="utf-8", errors="ignore")
        if "print(" in text:
            errs.append(f"{rel}: direct print() forbidden in ops command entrypoints; use commands.ops._shared.output")
    if errs:
        print("\n".join(errs))
        return 1
    print("ops command entrypoint print usage OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
