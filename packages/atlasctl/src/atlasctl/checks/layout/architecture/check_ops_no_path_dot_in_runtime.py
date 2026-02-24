#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[7]
OPS_CMDS = ROOT / "packages/atlasctl/src/atlasctl/commands/ops"
_PATH_DOT_DOUBLE = ('Path("', '.")')
_PATH_DOT_SINGLE = ("Path('", ".')")


def main() -> int:
    errs: list[str] = []
    for path in sorted(OPS_CMDS.rglob("*.py")):
        rel = path.relative_to(ROOT).as_posix()
        if "/tests/" in rel or "/internal/" in rel:
            continue
        text = path.read_text(encoding="utf-8", errors="ignore")
        if (_PATH_DOT_DOUBLE[0] + _PATH_DOT_DOUBLE[1]) in text or (_PATH_DOT_SINGLE[0] + _PATH_DOT_SINGLE[1]) in text:
            errs.append(f"{rel}: forbidden dot-path usage in ops runtime/commands; resolve through repo_root")
    if errs:
        print("\n".join(errs))
        return 1
    print("ops runtime dot-path usage OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
