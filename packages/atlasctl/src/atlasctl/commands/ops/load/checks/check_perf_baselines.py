#!/usr/bin/env python3
from __future__ import annotations

import sys
import json
import hashlib
import datetime as dt
from pathlib import Path


def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for base in (cur, *cur.parents):
        if all((base / part).exists() for part in ("makefiles", "packages", "configs", "ops")):
            return base
    raise RuntimeError("unable to resolve repository root")


def _run_contract(repo_root: Path) -> tuple[int, list[str]]:
    baselines_dir = repo_root / "configs" / "ops" / "perf" / "baselines"
    schema = json.loads((repo_root / "ops" / "schema" / "load" / "perf-baseline.schema.json").read_text(encoding="utf-8"))
    budgets = json.loads((repo_root / "configs" / "ops" / "budgets.json").read_text(encoding="utf-8"))
    tools = json.loads((repo_root / "configs" / "ops" / "tool-versions.json").read_text(encoding="utf-8"))
    lock = repo_root / "ops" / "datasets" / "manifest.lock"
    expected_lock = hashlib.sha256(lock.read_bytes()).hexdigest()[:16]
    req = set(schema.get("required", []))
    freshness_days = int(budgets.get("k6_latency", {}).get("baseline_freshness_days", 30))
    warn_only = bool(budgets.get("k6_latency", {}).get("baseline_freshness_warn_only", True))
    today = dt.date.today()
    errors: list[str] = []
    found = sorted(baselines_dir.glob("*.json"))
    if not found:
        errors.append(f"no baselines in {baselines_dir.relative_to(repo_root)}")
    for path in found:
        data = json.loads(path.read_text(encoding="utf-8"))
        ctx = path.relative_to(repo_root).as_posix()
        for key in req:
            if key not in data:
                errors.append(f"{ctx}: missing `{key}`")
        meta = data.get("metadata", {})
        for key in ("environment", "profile", "dataset_set", "dataset_lock_hash", "k8s_profile", "replicas", "tool_versions", "captured_at"):
            if key not in meta:
                errors.append(f"{ctx}.metadata: missing `{key}`")
        try:
            captured = dt.datetime.fromisoformat(str(meta.get("captured_at", "")).replace("Z", "+00:00")).date()
            age_days = (today - captured).days
            if age_days > freshness_days and not warn_only:
                errors.append(f"{ctx}: baseline older than {freshness_days} days ({age_days}d)")
        except Exception:
            errors.append(f"{ctx}: invalid metadata.captured_at")
        if str(meta.get("dataset_lock_hash", "")) != expected_lock:
            errors.append(f"{ctx}: dataset_lock_hash mismatch (expected {expected_lock})")
        tv = meta.get("tool_versions", {})
        for key in ("k6", "kind", "kubectl", "helm"):
            expected = str(tools.get(key, ""))
            got = str(tv.get(key, ""))
            if expected and got and expected != got:
                errors.append(f"{ctx}: tool version mismatch {key}: baseline={got} expected={expected}")
    return (0 if not errors else 1), (["perf baseline contract check passed"] if not errors else errors)


def main() -> int:
    repo_root = _repo_root()
    code, rows = _run_contract(repo_root)
    stream = sys.stderr if code else sys.stdout
    for row in rows:
        print(row, file=stream)
    return code


if __name__ == "__main__":
    raise SystemExit(main())
