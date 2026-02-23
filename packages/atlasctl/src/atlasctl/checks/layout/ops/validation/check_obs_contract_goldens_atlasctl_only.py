#!/usr/bin/env python3
from __future__ import annotations

import json
import sys
from pathlib import Path


def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for parent in cur.parents:
        if all((parent / marker).exists() for marker in ("ops", "packages", "configs", "makefiles")):
            return parent
    raise RuntimeError("unable to resolve repo root")


ROOT = _repo_root()


def main() -> int:
    errors: list[str] = []
    # Stable ordering for JSON goldens.
    for path in sorted((ROOT / "ops/obs/contract").rglob("*golden*.json")):
        try:
            obj = json.loads(path.read_text(encoding="utf-8"))
        except Exception as exc:  # pragma: no cover
            errors.append(f"invalid json golden {path.relative_to(ROOT)}: {exc}")
            continue
        normalized = json.dumps(obj, indent=2, sort_keys=True) + "\n"
        if path.read_text(encoding="utf-8") != normalized:
            errors.append(f"obs golden json must be deterministically sorted/formatted: {path.relative_to(ROOT)}")
    # Generated only via atlasctl in make/workflow surfaces.
    for rel in [Path("makefiles"), Path(".github/workflows")]:
        for file in rel.rglob("*"):
            if not file.is_file():
                continue
            txt = file.read_text(encoding="utf-8", errors="replace")
            if "ops/obs/contract/goldens" in txt and "atlasctl" not in txt:
                errors.append(f"direct obs contract golden reference outside atlasctl wrapper: {file}")
    if errors:
        print("obs contract golden policy check failed:", file=sys.stderr)
        for err in errors:
            print(f"- {err}", file=sys.stderr)
        return 1
    print("obs contract goldens policy passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
