#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[7]
CLI_MAIN = ROOT / "packages/atlasctl/src/atlasctl/cli/main.py"


def main() -> int:
    text = CLI_MAIN.read_text(encoding="utf-8", errors="ignore")
    required = (
        'parser.add_argument("--profile"',
        'parser.add_argument("--allow-network"',
        'parser.add_argument("--network",',
    )
    missing = [tok for tok in required if tok not in text]
    if missing:
        print("missing required global cli flags for ops profile/network consistency:")
        for tok in missing:
            print(f"- {tok}")
        return 1
    print("cli profile/network flags OK for ops command groups")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
