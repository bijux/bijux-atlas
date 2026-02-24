#!/usr/bin/env python3
from __future__ import annotations

import sys
import json
from pathlib import Path


def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for base in (cur, *cur.parents):
        if all((base / part).exists() for part in ("makefiles", "packages", "configs", "ops")):
            return base
    raise RuntimeError("unable to resolve repository root")


def _run_contract(repo_root: Path) -> tuple[int, list[str]]:
    manifest = json.loads((repo_root / "ops" / "load" / "suites" / "suites.json").read_text(encoding="utf-8"))
    runbook = (repo_root / "docs" / "operations" / "runbooks" / "load-failure-triage.md").read_text(encoding="utf-8")
    missing = [str(s.get("name")) for s in manifest.get("suites", []) if isinstance(s, dict) and s.get("name") and str(s.get("name")) not in runbook]
    if missing:
        return 1, [f"runbook missing suite name: {name}" for name in missing]
    return 0, ["runbook suite-name coverage passed"]


def main() -> int:
    repo_root = _repo_root()
    code, rows = _run_contract(repo_root)
    stream = sys.stderr if code else sys.stdout
    for row in rows:
        print(row, file=stream)
    return code


if __name__ == "__main__":
    raise SystemExit(main())
