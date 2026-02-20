#!/usr/bin/env python3
from __future__ import annotations

import argparse
import shutil
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]
SAFE_DIRS = [
    ROOT / "ops" / "_evidence" / "make",
    ROOT / "artifacts" / "isolate",
    ROOT / "artifacts" / "ops",
]


def wipe(path: Path) -> None:
    if not path.exists():
        return
    for child in path.iterdir():
        if child.is_dir():
            shutil.rmtree(child)
        else:
            child.unlink()


def main() -> int:
    p = argparse.ArgumentParser()
    p.add_argument("--all", action="store_true")
    args = p.parse_args()

    targets = SAFE_DIRS if args.all else [ROOT / "ops" / "_evidence" / "make"]
    for target in targets:
        target.mkdir(parents=True, exist_ok=True)
        wipe(target)
        print(target.relative_to(ROOT))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
