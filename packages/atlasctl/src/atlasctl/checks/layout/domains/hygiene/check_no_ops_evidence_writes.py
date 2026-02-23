#!/usr/bin/env python3
from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[6]

SCAN_ROOTS = (
    ROOT / "ops" / "run",
    ROOT / "ops" / "_lib",
    ROOT / "makefiles",
)

ALLOWED_TOKENS = (
    "ops/_generated.example/",
    "ops/_generated_static/",
)

FORBIDDEN_TOKENS = (
    "ops/_evidence/",
    "ops/_generated/",
)

SUFFIXES = {".sh", ".mk"}


def should_scan(path: Path) -> bool:
    if path.suffix not in SUFFIXES:
        return False
    rel = path.relative_to(ROOT).as_posix()
    if rel.startswith("ops/_evidence/"):
        return False
    if rel.startswith("ops/_generated/"):
        return False
    if rel.startswith("ops/_generated.example/"):
        return False
    return True


def main() -> int:
    errors: list[str] = []
    for base in SCAN_ROOTS:
        if not base.exists():
            continue
        for path in sorted(p for p in base.rglob("*") if p.is_file() and should_scan(p)):
            rel = path.relative_to(ROOT).as_posix()
            for lineno, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
                if line.lstrip().startswith("#"):
                    continue
                for token in FORBIDDEN_TOKENS:
                    if token not in line:
                        continue
                    if any(allowed in line for allowed in ALLOWED_TOKENS):
                        continue
                    errors.append(f"{rel}:{lineno}: forbidden runtime evidence path `{token}`; use `artifacts/evidence/`")

    if errors:
        print("ops evidence write-path check failed", file=sys.stderr)
        for item in errors[:200]:
            print(f"- {item}", file=sys.stderr)
        return 1

    print("ops evidence write-path check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
