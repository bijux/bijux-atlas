from __future__ import annotations

import sys
from pathlib import Path

if __package__ in (None, ""):
    sys.path.insert(0, str(Path(__file__).resolve().parents[6]))

from atlasctl.commands.ops.e2e.realdata._common import atlasctl


def main(argv: list[str] | None = None) -> int:
    args = list(sys.argv[1:] if argv is None else argv)
    atlasctl("ops", "e2e", "run", "--suite", "realdata", *args)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
