#!/usr/bin/env python3
from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]


def fail(msg: str) -> int:
    print(f"lane isolation check failed: {msg}", file=sys.stderr)
    return 1


def main() -> int:
    if len(sys.argv) < 3:
        print("usage: check_root_local_lane_isolation.py <run_id> <lane> [<lane> ...]", file=sys.stderr)
        return 2

    run_id = sys.argv[1]
    lanes = sys.argv[2:]

    tmp_dirs: list[Path] = []
    for lane in lanes:
        iso = ROOT / "artifacts" / "isolate" / lane / run_id
        tmp = iso / "tmp"
        target = iso / "target"
        cargo_home = iso / "cargo-home"

        for required in (iso, tmp, target, cargo_home):
            if not required.exists():
                return fail(f"missing lane isolate path for {lane}: {required}")

        resolved = tmp.resolve()
        if "artifacts/isolate" not in resolved.as_posix():
            return fail(f"tmp dir escapes artifacts/isolate for {lane}: {resolved}")

        tmp_dirs.append(resolved)

    if len(set(tmp_dirs)) != len(tmp_dirs):
        return fail("duplicate tmp directories detected across lanes")

    print("lane isolation check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
