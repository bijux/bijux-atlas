from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[8]


def main() -> int:
    errs: list[str] = []
    for path in sorted(ROOT.rglob("docker-compose*.yml")):
        rel = path.relative_to(ROOT).as_posix()
        if not rel.startswith("ops/"):
            continue
        text = path.read_text(encoding="utf-8", errors="ignore")
        for m in re.finditer(r"^\s*image:\s*['\"]?([^'\"\s#]+)", text, re.MULTILINE):
            image = m.group(1).strip()
            if ":latest" in image:
                errs.append(f"{rel}: image uses :latest -> {image}")
            if "@sha256:" not in image:
                errs.append(f"{rel}: image not digest-pinned -> {image}")
    if errs:
        print("product compose pinning policy failed:", file=sys.stderr)
        for e in errs[:200]:
            print(e, file=sys.stderr)
        return 1
    print("product compose pinning policy check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
