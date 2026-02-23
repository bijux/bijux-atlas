from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[7]


def main() -> int:
    required_package_docs = [
        "packages/atlasctl/docs/control-plane/ops-execution-model.md",
        "packages/atlasctl/docs/control-plane/ops-taxonomy.md",
        "packages/atlasctl/docs/control-plane/ops-local-run.md",
        "packages/atlasctl/docs/control-plane/ops-ci-lanes.md",
        "packages/atlasctl/docs/commands/groups/ops.md",
    ]
    required_repo_docs = [
        "docs/_generated/ops-actions.md",
    ]
    errs: list[str] = []
    for rel in required_package_docs + required_repo_docs:
        p = ROOT / rel
        if not p.exists():
            errs.append(f"missing ops docs page: {rel}")
    if errs:
        print("ops docs nav integrity check failed:", file=sys.stderr)
        for e in errs:
            print(e, file=sys.stderr)
        return 1
    print("ops docs nav integrity check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
