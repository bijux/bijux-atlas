#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[7]
OPS_CMDS = ROOT / "packages/atlasctl/src/atlasctl/commands/ops"
FORBIDDEN = ("sys.stdout.write(",)
ALLOW = {
    "packages/atlasctl/src/atlasctl/commands/ops/_shared/output.py",  # central output layer
}


def main() -> int:
    errs: list[str] = []
    for path in sorted(OPS_CMDS.rglob("command.py")):
        rel = path.relative_to(ROOT).as_posix()
        if rel in ALLOW:
            continue
        text = path.read_text(encoding="utf-8", errors="ignore")
        for token in FORBIDDEN:
            if token in text:
                errs.append(f"{rel}: forbidden stdout/file write token `{token}` in command entrypoint")
    if errs:
        print("\n".join(errs))
        return 1
    print("ops command entrypoint stdout/file-write usage OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
