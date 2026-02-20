#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[3]


def main() -> int:
    seen: dict[str, list[str]] = {}
    errors: list[tuple[str, list[str]]] = []
    for path in sorted((ROOT / "scripts").rglob("*")):
        if not path.is_file() or path.suffix not in {".sh", ".py"}:
            continue
        canonical = path.stem.replace("_", "-")
        rel = path.relative_to(ROOT).as_posix()
        seen.setdefault(canonical, []).append(rel)

    for canonical, paths in seen.items():
        stems = {Path(p).stem for p in paths}
        if len(stems) > 1:
            errors.append((canonical, sorted(paths)))

    if errors:
        print("duplicate dash/underscore script names detected:", file=sys.stderr)
        for key, paths in errors:
            print(f"- {key}:", file=sys.stderr)
            for rel in paths:
                print(f"  - {rel}", file=sys.stderr)
        return 1
    print("no duplicate script names")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
