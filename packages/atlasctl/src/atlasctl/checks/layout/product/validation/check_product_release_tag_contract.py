from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[8]
COMMAND = ROOT / "packages/atlasctl/src/atlasctl/commands/product/command.py"
DOC = ROOT / "packages/atlasctl/docs/control-plane/product-release-tag-policy.md"


def main() -> int:
    errs: list[str] = []
    if not DOC.exists():
        errs.append("missing product release tag policy doc")
    else:
        text = DOC.read_text(encoding="utf-8", errors="ignore")
        for token in ("vMAJOR.MINOR.PATCH", "release-candidate", "GITHUB_REF_NAME", "RELEASE_TAG"):
            if token not in text:
                errs.append(f"release tag policy doc missing token: {token}")
    cmd = COMMAND.read_text(encoding="utf-8", errors="ignore")
    for token in ("_validate_release_tag_contract", "GITHUB_REF_NAME", "RELEASE_TAG", "tag does not match release tag contract"):
        if token not in cmd:
            errs.append(f"product command missing release tag contract enforcement token: {token}")
    if errs:
        print("product release tag contract check failed:", file=sys.stderr)
        for e in errs:
            print(e, file=sys.stderr)
        return 1
    print("product release tag contract check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
