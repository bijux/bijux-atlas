#!/usr/bin/env python3
# Purpose: validate help output determinism and ensure no forbidden legacy targets appear.
from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[6]
FORBIDDEN_RE = re.compile(r"(^|/)legacy($|-)")


def render(mode: str | None = None) -> str:
    cmd = ["bin/atlasctl", "make", "help"]
    if mode:
        cmd.extend(["--mode", mode])
    p = subprocess.run(cmd, cwd=ROOT, capture_output=True, text=True, check=False)
    if p.returncode != 0:
        raise RuntimeError(f"help renderer failed for mode={mode or 'help'}: {p.stderr or p.stdout}")
    return p.stdout


def main() -> int:
    try:
        help_1 = render()
        help_2 = render()
        adv = render("advanced")
    except RuntimeError as err:
        print(str(err), file=sys.stderr)
        return 1

    if help_1 != help_2:
        print("help snapshot check failed: `make help` output is not deterministic", file=sys.stderr)
        return 1

    offenders = []
    for name, payload in (("help", help_1), ("advanced", adv)):
        for line in payload.splitlines():
            token = line.strip().split(" ", 1)[0]
            if FORBIDDEN_RE.search(token):
                offenders.append(f"{name}: {token}")
    if offenders:
        print("help snapshot check failed: legacy targets leaked into help output", file=sys.stderr)
        for item in offenders:
            print(f"- {item}", file=sys.stderr)
        return 1

    print("help snapshot check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
