#!/usr/bin/env python3
from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[8]
if str(ROOT / "packages" / "atlasctl" / "src") not in sys.path:
    sys.path.insert(0, str(ROOT / "packages" / "atlasctl" / "src"))

from atlasctl.checks.tools.ops_domain.contracts import check_ops_surface_manifest_native


def main() -> int:
    code, errors = check_ops_surface_manifest_native(ROOT)
    if errors:
        print("ops surface manifest failed:", file=sys.stderr)
        for e in errors:
            print(e, file=sys.stderr)
        return code or 1
    print("ops surface manifest passed")
    return code


if __name__ == "__main__":
    raise SystemExit(main())
