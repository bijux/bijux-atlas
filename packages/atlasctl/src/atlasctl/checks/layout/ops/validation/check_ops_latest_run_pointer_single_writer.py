from __future__ import annotations

import sys
import re
from pathlib import Path


ROOT = Path(__file__).resolve().parents[8]
TARGET = "artifacts/evidence/latest-run-id.txt"
SEARCH = ROOT / "packages" / "atlasctl" / "src" / "atlasctl" / "commands" / "ops"
WRITE_RE = re.compile(r"(write_text\s*\(|printf\s+.*>\s*)[^\n]*artifacts/evidence/latest-run-id\.txt")


def main() -> int:
    writers: list[str] = []
    for path in sorted(SEARCH.rglob("*")):
        if not path.is_file():
            continue
        if path.suffix not in {".py", ".sh"}:
            continue
        text = path.read_text(encoding="utf-8", errors="ignore")
        if TARGET in text and WRITE_RE.search(text):
            writers.append(path.relative_to(ROOT).as_posix())
    # de-dup and enforce exactly one writer in ops command/assets surface
    writers = sorted(set(writers))
    if len(writers) != 1:
        print(f"expected exactly one writer of {TARGET}, found {len(writers)}", file=sys.stderr)
        for item in writers:
            print(f"- {item}", file=sys.stderr)
        return 1
    print(f"single latest-run-id pointer writer: {writers[0]}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
