from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[8]

SCAN_GLOBS = [
    "ops/**/*.py",
    "ops/**/*.sh",
    "scripts/**/*.py",
    "scripts/**/*.sh",
]


def main() -> int:
    errs: list[str] = []
    pat = re.compile(r"\bkubectl\s+apply\b")
    for glob in SCAN_GLOBS:
        for path in sorted(ROOT.glob(glob)):
            if not path.is_file():
                continue
            rel = path.relative_to(ROOT).as_posix()
            text = path.read_text(encoding="utf-8", errors="ignore")
            if pat.search(text):
                errs.append(f"{rel}: direct 'kubectl apply' is forbidden; use atlasctl runner with reporting")
    if errs:
        print("product script kubectl-apply policy failed:", file=sys.stderr)
        for e in errs[:200]:
            print(e, file=sys.stderr)
        return 1
    print("product script kubectl-apply policy check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
