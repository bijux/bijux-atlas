#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[7]
OPS_CMDS = ROOT / "packages/atlasctl/src/atlasctl/commands/ops"


def main() -> int:
    errs: list[str] = []
    for path in sorted(OPS_CMDS.rglob("*.py")):
        rel = path.relative_to(ROOT).as_posix()
        if "/tests/" in rel or "/internal/" in rel:
            continue
        text = path.read_text(encoding="utf-8", errors="ignore")
        if 'Path(".")' in text or "Path('.')" in text:
            errs.append(f"{rel}: forbidden Path('.') usage in ops runtime/commands; resolve through repo_root")
    if errs:
        print("\n".join(errs))
        return 1
    print("ops runtime Path('.') usage OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
