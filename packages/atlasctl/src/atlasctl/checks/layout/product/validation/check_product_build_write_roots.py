from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[8]
COMMAND = ROOT / "packages/atlasctl/src/atlasctl/commands/product/command.py"


def main() -> int:
    text = COMMAND.read_text(encoding="utf-8", errors="ignore")
    errs: list[str] = []
    for m in re.finditer(r'"(artifacts/[^"]+)"', text):
        path = m.group(1)
        if not (path.startswith("artifacts/chart") or path.startswith("artifacts/evidence/product")):
            errs.append(f"unexpected product write root literal: {path}")
    if "_product_manifest_path" not in text:
        errs.append("product command should centralize artifact manifest output path")
    if errs:
        print("product build write roots check failed:", file=sys.stderr)
        for e in errs:
            print(e, file=sys.stderr)
        return 1
    print("product build write roots check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
