#!/usr/bin/env python3
from __future__ import annotations

import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[8]


def main() -> int:
    profiles_doc = json.loads((ROOT / "ops/stack/profiles.json").read_text(encoding="utf-8"))
    matrix = json.loads((ROOT / "ops/k8s/install-matrix.json").read_text(encoding="utf-8"))
    stack_profiles = sorted({str(item.get("name")) for item in profiles_doc.get("profiles", []) if isinstance(item, dict) and item.get("name")})
    matrix_profiles = sorted(
        {
            str(item.get("name"))
            for item in matrix.get("profiles", [])
            if isinstance(item, dict) and item.get("name")
        }
    )

    errs: list[str] = []
    # The install matrix is a chart-values profile list, not a 1:1 stack runtime profile list.
    # Enforce the shared bridge contract: perf profile must exist in both, because perf stack and perf chart
    # profile need to stay aligned for load/soak lanes.
    if "perf" in matrix_profiles and "perf" not in stack_profiles:
        errs.append("ops/stack/profiles.json missing `perf` profile required by ops/k8s/install-matrix.json")
    if "perf" in stack_profiles and "perf" not in matrix_profiles:
        errs.append("ops/k8s/install-matrix.json missing `perf` profile required by ops/stack/profiles.json")
    if errs:
        print("\n".join(errs), file=sys.stderr)
        return 1
    print("stack/runtime and install-matrix shared profile contract is aligned")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
