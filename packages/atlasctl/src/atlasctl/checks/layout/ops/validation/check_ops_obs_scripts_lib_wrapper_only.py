from __future__ import annotations

import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[8]
LIB_DIR = ROOT / "ops" / "obs" / "scripts" / "lib"


def main() -> int:
    if not LIB_DIR.exists():
        print("ops/obs/scripts/lib does not exist")
        return 0

    errs: list[str] = []
    for path in sorted(LIB_DIR.rglob("*")):
        if not path.is_file():
            continue
        rel = path.relative_to(ROOT).as_posix()
        if path.suffix.lower() not in {".md", ".txt"}:
            errs.append(f"{rel}: direct runnable/lib script files under ops/obs/scripts/lib are forbidden")

    # Disallow makefile direct references to the lib path.
    for mk in sorted((ROOT / "makefiles").rglob("*.mk")):
        text = mk.read_text(encoding="utf-8", errors="ignore")
        if "ops/obs/scripts/lib/" in text:
            errs.append(f"{mk.relative_to(ROOT).as_posix()}: makefiles must not reference ops/obs/scripts/lib directly")

    if errs:
        print("\n".join(errs), file=sys.stderr)
        return 1
    print("ops/obs/scripts/lib is wrapper-only (no direct runnable files or make references)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
