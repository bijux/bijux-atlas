#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from pathlib import Path

def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for base in (cur, *cur.parents):
        if (base / "makefiles").exists() and (base / "packages").exists() and (base / ".github").exists():
            return base
    raise RuntimeError("unable to resolve repository root")


ROOT = _repo_root()
CI_MK = ROOT / "makefiles" / "ci.mk"

# CI wrappers must not embed direct filesystem writes; atlasctl owns writes.
FORBIDDEN_WRITE_PATTERNS = (
    re.compile(r">+\s*(?!artifacts/(?:isolate|evidence)/)"),
    re.compile(r"\bmkdir\s+-p\s+(?!\"?artifacts/(?:isolate|evidence)/)"),
    re.compile(r"\bcp\s+"),
    re.compile(r"\bmv\s+"),
    re.compile(r"\brm\s+"),
)


def main() -> int:
    errors: list[str] = []
    for idx, line in enumerate(CI_MK.read_text(encoding="utf-8").splitlines(), start=1):
        if not line.startswith("\t"):
            continue
        body = line.strip()
        for pattern in FORBIDDEN_WRITE_PATTERNS:
            if pattern.search(body):
                errors.append(f"makefiles/ci.mk:{idx}: ci wrappers must not perform direct writes: `{body}`")
                break
    if errors:
        print("ci write-scope check failed", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1
    print("ci write-scope check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
