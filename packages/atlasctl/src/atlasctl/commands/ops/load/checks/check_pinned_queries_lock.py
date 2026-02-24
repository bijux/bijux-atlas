#!/usr/bin/env python3
from __future__ import annotations

import sys
import json
import hashlib
from pathlib import Path


def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for base in (cur, *cur.parents):
        if all((base / part).exists() for part in ("makefiles", "packages", "configs", "ops")):
            return base
    raise RuntimeError("unable to resolve repository root")


def _run_contract(repo_root: Path) -> tuple[int, list[str]]:
    src = repo_root / "ops" / "load" / "queries" / "pinned-v1.json"
    lock_path = repo_root / "ops" / "load" / "queries" / "pinned-v1.lock"
    schema_path = repo_root / "ops" / "schema" / "load" / "pinned-queries-lock.schema.json"
    suites_manifest_path = repo_root / "ops" / "load" / "suites" / "suites.json"
    queries = json.loads(src.read_text(encoding="utf-8"))
    lock = json.loads(lock_path.read_text(encoding="utf-8"))
    schema = json.loads(schema_path.read_text(encoding="utf-8"))
    suites_manifest = json.loads(suites_manifest_path.read_text(encoding="utf-8"))
    errors: list[str] = []
    if not isinstance(lock, dict):
        return 1, ["pinned query lock must be object"]
    for key in schema.get("required", []):
        if key not in lock:
            errors.append(f"pinned query lock missing required key: {key}")
    file_hash = hashlib.sha256(src.read_bytes()).hexdigest()
    if lock.get("file_sha256") != file_hash:
        errors.append("pinned query lock drift: file hash mismatch")
    expected: dict[str, str] = {}
    for group in ("cheap", "heavy"):
        for query in queries.get(group, []):
            expected[query] = hashlib.sha256(query.encode()).hexdigest()
    if lock.get("query_hashes") != expected:
        errors.append("pinned query lock drift: query hash mismatch")
    query_set = str(suites_manifest.get("query_set", "")).strip()
    if Path(query_set).name != "pinned-v1.json":
        errors.append(f"load suites manifest query_set must reference pinned-v1.json (got `{query_set}`)")
    return (0 if not errors else 1), (["pinned query lock check passed"] if not errors else errors)


def main() -> int:
    repo_root = _repo_root()
    code, rows = _run_contract(repo_root)
    stream = sys.stderr if code else sys.stdout
    for row in rows:
        print(row, file=stream)
    return code


if __name__ == "__main__":
    raise SystemExit(main())
