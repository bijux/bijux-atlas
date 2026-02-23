from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[8]
GEN = ROOT / "packages/atlasctl/src/atlasctl/commands/dev/gen/command.py"


def main() -> int:
    text = GEN.read_text(encoding="utf-8", errors="ignore")
    required = ['sub == "goldens-update"', "gen goldens-update", "checks-registry", "--i-know-what-im-doing"]
    missing = [t for t in required if t not in text]
    if missing:
        print("goldens update workflow command check failed:", file=sys.stderr)
        for t in missing:
            print(f"missing token: {t}", file=sys.stderr)
        return 1
    print("goldens update workflow command check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
