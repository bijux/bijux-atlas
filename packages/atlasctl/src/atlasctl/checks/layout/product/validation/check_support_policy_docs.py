from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[8]


def main() -> int:
    doc = ROOT / "packages/atlasctl/docs/control-plane/support-policy.md"
    errs: list[str] = []
    if not doc.exists():
        errs.append("missing support-policy.md")
    else:
        text = doc.read_text(encoding="utf-8", errors="ignore")
        for token in ("atlasctl ops", "atlasctl product", "deterministic reports", "schema-validated outputs"):
            if token not in text:
                errs.append(f"support-policy.md missing token: {token}")
    if errs:
        print("support policy docs check failed:", file=sys.stderr)
        for e in errs:
            print(e, file=sys.stderr)
        return 1
    print("support policy docs check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
