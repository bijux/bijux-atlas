#!/usr/bin/env python3
from __future__ import annotations

import datetime as dt
import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
BASELINES = ROOT / "configs/ops/perf/baselines"
SCHEMA = ROOT / "ops/_schemas/load/perf-baseline.schema.json"
BUDGETS = ROOT / "configs/ops/budgets.json"
TOOLS = ROOT / "configs/ops/tool-versions.json"
LOCK = ROOT / "ops/datasets/manifest.lock"


def _req(obj: dict, key: str, errs: list[str], ctx: str) -> None:
    if key not in obj:
        errs.append(f"{ctx}: missing `{key}`")


def _lock_hash() -> str:
    return hashlib.sha256(LOCK.read_bytes()).hexdigest()[:16]


def main() -> int:
    errs: list[str] = []
    schema = json.loads(SCHEMA.read_text(encoding="utf-8"))
    req = set(schema.get("required", []))
    budgets = json.loads(BUDGETS.read_text(encoding="utf-8"))
    tools = json.loads(TOOLS.read_text(encoding="utf-8"))
    expected_lock = _lock_hash()
    freshness_days = int(budgets.get("k6_latency", {}).get("baseline_freshness_days", 30))
    warn_only = bool(budgets.get("k6_latency", {}).get("baseline_freshness_warn_only", True))
    today = dt.date.today()

    found = sorted(BASELINES.glob("*.json"))
    if not found:
        errs.append(f"no baselines in {BASELINES.relative_to(ROOT)}")

    for path in found:
        data = json.loads(path.read_text(encoding="utf-8"))
        ctx = path.relative_to(ROOT).as_posix()
        for k in req:
            _req(data, k, errs, ctx)
        meta = data.get("metadata", {})
        for k in ("environment", "profile", "dataset_set", "dataset_lock_hash", "k8s_profile", "replicas", "tool_versions", "captured_at"):
            _req(meta, k, errs, f"{ctx}.metadata")
        try:
            captured = dt.datetime.fromisoformat(str(meta.get("captured_at", "")).replace("Z", "+00:00")).date()
            age_days = (today - captured).days
            if age_days > freshness_days:
                msg = f"{ctx}: baseline older than {freshness_days} days ({age_days}d)"
                if warn_only:
                    print(f"warning: {msg}")
                else:
                    errs.append(msg)
        except Exception:  # noqa: BLE001
            errs.append(f"{ctx}: invalid metadata.captured_at")

        if str(meta.get("dataset_lock_hash", "")) != expected_lock:
            errs.append(f"{ctx}: dataset_lock_hash mismatch (expected {expected_lock})")

        tv = meta.get("tool_versions", {})
        for key in ("k6", "kind", "kubectl", "helm"):
            expected = str(tools.get(key, ""))
            got = str(tv.get(key, ""))
            if expected and got and expected != got:
                errs.append(f"{ctx}: tool version mismatch {key}: baseline={got} expected={expected}")

    if errs:
        print("perf baseline contract check failed", file=sys.stderr)
        for e in errs:
            print(f"- {e}", file=sys.stderr)
        return 1
    print("perf baseline contract check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
