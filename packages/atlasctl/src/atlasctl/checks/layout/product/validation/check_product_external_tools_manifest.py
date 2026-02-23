from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[8]
MANIFEST = ROOT / "configs/product/external-tools-manifest.json"
COMMAND = ROOT / "packages/atlasctl/src/atlasctl/commands/product/command.py"


def main() -> int:
    errs: list[str] = []
    if not MANIFEST.exists():
        errs.append("missing configs/product/external-tools-manifest.json")
    else:
        try:
            payload = json.loads(MANIFEST.read_text(encoding="utf-8"))
        except Exception as exc:
            errs.append(f"invalid product external tools manifest json: {exc}")
            payload = {}
        tools = payload.get("tools")
        if not isinstance(tools, list) or not all(isinstance(x, str) and x.strip() for x in tools):
            errs.append("manifest tools[] must be non-empty strings")
        workflows = payload.get("workflows")
        if not isinstance(workflows, dict) or not workflows:
            errs.append("manifest workflows{} is required")
    text = COMMAND.read_text(encoding="utf-8", errors="ignore")
    for token in ("PRODUCT_TOOLS_MANIFEST", "_validate_product_tools_manifest", "release gate failed: product external tools manifest invalid"):
        if token not in text:
            errs.append(f"missing product external-tools enforcement token: {token}")
    if errs:
        print("product external tools manifest check failed:", file=sys.stderr)
        for e in errs:
            print(e, file=sys.stderr)
        return 1
    print("product external tools manifest check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
