from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[8]
GEN = ROOT / "packages/atlasctl/src/atlasctl/commands/dev/gen/command.py"


def main() -> int:
    text = GEN.read_text(encoding="utf-8", errors="ignore")
    for token in ('sub == "release-notes"', "ci\", \"release-notes-render", "--i-know-what-im-doing"):
        if token not in text:
            print(f"missing release-notes generator token: {token}", file=sys.stderr)
            return 1
    print("product release-notes generator check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
