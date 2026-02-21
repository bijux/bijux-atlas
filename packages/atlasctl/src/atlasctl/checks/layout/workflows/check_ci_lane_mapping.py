#!/usr/bin/env python3
# Purpose: ensure ci.mk ci-* wrappers map 1:1 to atlasctl ci lane registry.
from __future__ import annotations

import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[7]
CI_MK = ROOT / "makefiles" / "ci.mk"
TARGET_RE = re.compile(r"^([A-Za-z0-9_./-]+):(?:\s|$)", re.M)


LANE_TARGETS = {
    "ci",
    "ci-fast",
    "ci-all",
    "ci-contracts",
    "ci-docs",
    "ci-ops",
    "ci-release",
    "ci-release-all",
}


def _ci_wrapper_targets() -> set[str]:
    text = CI_MK.read_text(encoding="utf-8")
    out = set()
    for target in TARGET_RE.findall(text):
        if target in LANE_TARGETS:
            out.add(target)
    return out


def _ci_lanes() -> tuple[set[str], list[str]]:
    proc = subprocess.run(
        ["./bin/atlasctl", "--quiet", "ci", "list", "--json"],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    if proc.returncode != 0:
        raise RuntimeError(proc.stderr or proc.stdout or "failed to run atlasctl ci list")
    payload = json.loads(proc.stdout)
    lanes = {str(row["name"]) for row in payload.get("lanes", []) if str(row.get("kind", "")) == "lane"}
    suite_errors = [str(row.get("name", "")) for row in payload.get("lanes", []) if str(row.get("kind", "")) == "lane" and not str(row.get("suite", "")).strip()]
    return lanes, suite_errors


def main() -> int:
    try:
        wrappers = _ci_wrapper_targets()
        lanes, suite_errors = _ci_lanes()
    except RuntimeError as exc:
        print(f"ci lane mapping check failed: {exc}", file=sys.stderr)
        return 1

    missing = sorted(wrappers - lanes)
    extra = sorted(lanes - wrappers)
    if missing or extra or suite_errors:
        print("ci lane mapping check failed", file=sys.stderr)
        for name in missing:
            print(f"- ci.mk wrapper missing from atlasctl ci list: {name}", file=sys.stderr)
        for name in extra:
            print(f"- atlasctl ci lane missing wrapper in ci.mk: {name}", file=sys.stderr)
        for name in suite_errors:
            print(f"- atlasctl ci lane missing suite mapping: {name}", file=sys.stderr)
        return 1
    print("ci lane mapping check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
