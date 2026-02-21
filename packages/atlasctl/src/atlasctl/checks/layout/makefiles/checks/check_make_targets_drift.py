#!/usr/bin/env python3
# Purpose: enforce make public target drift against SSOT surface.
from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[8]
SSOT = ROOT / "configs" / "make" / "public-targets.json"
CATALOG = ROOT / "makefiles" / "targets.json"
TARGET_RE = re.compile(r"^([A-Za-z0-9_./-]+):(?:\s|$)", re.M)


def _targets_from_make(path: Path) -> set[str]:
    text = path.read_text(encoding="utf-8")
    return {target for target in TARGET_RE.findall(text) if not target.startswith(".")}


def main() -> int:
    ssot = json.loads(SSOT.read_text(encoding="utf-8"))
    allowed = {str(item.get("name", "")).strip() for item in ssot.get("public_targets", []) if str(item.get("name", "")).strip()}
    catalog = json.loads(CATALOG.read_text(encoding="utf-8"))
    catalog_targets = {str(item.get("name", "")).strip() for item in catalog.get("targets", []) if str(item.get("name", "")).strip()}
    published = _targets_from_make(ROOT / "makefiles" / "root.mk") | _targets_from_make(ROOT / "makefiles" / "product.mk")

    errors: list[str] = []
    if allowed != catalog_targets:
        errors.append("makefiles/targets.json drift vs configs/make/public-targets.json")
    missing_from_publication = sorted(allowed - published)
    if missing_from_publication:
        errors.append(f"public targets missing from root/product publication: {', '.join(missing_from_publication)}")

    if errors:
        print("make targets drift check failed", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1

    print("make targets drift check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
