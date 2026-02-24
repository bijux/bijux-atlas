#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[7]
OPS_CMDS = ROOT / "packages/atlasctl/src/atlasctl/commands/ops"
FORBIDDEN_TOKENS = ("os.chdir(", "cwd='.'", 'cwd="."', "cwd=Path(")


def main() -> int:
    errs: list[str] = []
    for path in sorted(OPS_CMDS.rglob("*.py")):
        rel = path.relative_to(ROOT).as_posix()
        if "/tests/" in rel or "/internal/" in rel:
            continue
        text = path.read_text(encoding="utf-8", errors="ignore")
        for token in FORBIDDEN_TOKENS:
            if token in text:
                errs.append(f"{rel}: cwd-reliance token forbidden (`{token}`); use repo_root + explicit paths")
    if errs:
        print("\n".join(errs))
        return 1
    print("ops cwd reliance check OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
