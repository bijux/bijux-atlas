#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
BIN = ROOT / "scripts/bin"


def main() -> int:
    errors: list[str] = []
    for p in sorted(BIN.glob("*")):
        if not p.is_file():
            continue
        text = p.read_text(encoding="utf-8", errors="ignore")
        if "#!/usr/bin/env" not in text:
            errors.append(f"{p.relative_to(ROOT)} missing shebang")
        if "purpose:" not in text.lower() and "usage:" not in text.lower():
            errors.append(f"{p.relative_to(ROOT)} missing purpose/usage header")

    if errors:
        print("script entrypoint policy failed:", file=sys.stderr)
        for err in errors:
            print(f"- {err}", file=sys.stderr)
        return 1

    print("script entrypoint policy passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
