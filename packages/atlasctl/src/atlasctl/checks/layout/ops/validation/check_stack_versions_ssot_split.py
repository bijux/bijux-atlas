#!/usr/bin/env python3
from __future__ import annotations

import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[8]


def main() -> int:
    versions = json.loads((ROOT / "ops/stack/versions.json").read_text(encoding="utf-8"))
    manifest = json.loads((ROOT / "ops/stack/version-manifest.json").read_text(encoding="utf-8"))

    errs: list[str] = []
    tools = versions.get("tools")
    if not isinstance(tools, dict):
        errs.append("ops/stack/versions.json must contain object key `tools`")
        tools = {}
    if not isinstance(manifest, dict):
        errs.append("ops/stack/version-manifest.json must be an object")
        manifest = {}

    overlap = sorted(set(tools.keys()) & set(manifest.keys()))
    if overlap:
        errs.append(f"stack versions SSOT split violated; overlapping keys between versions.json.tools and version-manifest.json: {', '.join(overlap)}")

    if not manifest:
        errs.append("ops/stack/version-manifest.json must not be empty")
    if not any(isinstance(v, str) and ":" in v for v in manifest.values()):
        errs.append("ops/stack/version-manifest.json must contain image refs (tag or digest)")

    if errs:
        print("\n".join(errs), file=sys.stderr)
        return 1
    print("stack version SSOT split valid: version-manifest=image refs, versions.json.tools=tool versions")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

