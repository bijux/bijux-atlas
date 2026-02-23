from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[8]
COMMAND = ROOT / "packages/atlasctl/src/atlasctl/commands/product/command.py"


def main() -> int:
    text = COMMAND.read_text(encoding="utf-8", errors="ignore")
    errs: list[str] = []
    required = [
        '["./bin/atlasctl", "ops", "pins", "check", "--report", "text"]',
        "product validation requires pinned tool versions to pass",
    ]
    for token in required:
        if token not in text:
            errs.append(f"missing pinned-tool validation token in product command: {token}")
    if errs:
        print("product pinned tools policy check failed:", file=sys.stderr)
        for e in errs:
            print(e, file=sys.stderr)
        return 1
    print("product pinned tools policy check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
