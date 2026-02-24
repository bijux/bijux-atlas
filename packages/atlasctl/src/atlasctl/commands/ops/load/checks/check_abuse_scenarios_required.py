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
    suites = repo_root / "ops" / "load" / "suites" / "suites.json"
    payload = json.loads(suites.read_text(encoding="utf-8"))
    by_name = {s.get("name"): s for s in payload.get("suites", []) if isinstance(s, dict)}
    errors: list[str] = []
    abuse = by_name.get("response-size-abuse")
    if not abuse:
        errors.append("missing required suite: response-size-abuse")
    else:
        run_in = set(abuse.get("run_in", []))
        for profile in ("full", "nightly", "load-nightly"):
            if profile not in run_in:
                errors.append(f"response-size-abuse must run in {profile} profile")
        if not abuse.get("must_pass", False):
            errors.append("response-size-abuse must have must_pass=true")
    return (0 if not errors else 1), (["abuse scenario contract passed"] if not errors else errors)


def main() -> int:
    repo_root = _repo_root()
    code, rows = _run_contract(repo_root)
    stream = sys.stderr if code else sys.stdout
    for row in rows:
        print(row, file=stream)
    return code


if __name__ == "__main__":
    raise SystemExit(main())
