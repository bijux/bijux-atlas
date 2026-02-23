from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[8]
DOC = ROOT / "packages/atlasctl/docs/control-plane/schema-version-bump-playbook.md"


def main() -> int:
    if not DOC.exists():
        print("missing schema version bump playbook doc", file=sys.stderr)
        return 1
    text = DOC.read_text(encoding="utf-8", errors="ignore")
    required = [
        "new versioned schema file",
        "release-notes.md",
        "check run --group contracts",
    ]
    missing = [t for t in required if t not in text]
    if missing:
        print("schema version bump playbook check failed:", file=sys.stderr)
        for t in missing:
            print(f"missing token in playbook doc: {t}", file=sys.stderr)
        return 1
    print("schema version bump playbook check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
