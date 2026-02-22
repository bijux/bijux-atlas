#!/usr/bin/env python3
from __future__ import annotations

import json
import re
import sys
from pathlib import Path


def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for parent in cur.parents:
        if all((parent / marker).exists() for marker in ("makefiles", "packages", "configs", "ops")):
            return parent
    raise RuntimeError("unable to resolve repo root")


ROOT = _repo_root()
manifest_path = ROOT / "ops/load/suites/suites.json"
schema_path = ROOT / "ops/load/contracts/suite-schema.json"
if (ROOT / "ops/_schemas/load/suite-manifest.schema.json").exists():
    schema_path = ROOT / "ops/_schemas/load/suite-manifest.schema.json"
manifest = json.loads(manifest_path.read_text())
schema = json.loads(schema_path.read_text())

errors: list[str] = []

for key in schema.get("required", []):
    if key not in manifest:
        errors.append(f"missing required root key: {key}")

suites = manifest.get("suites", [])
seen = set()
name_re = re.compile(r"^[a-z0-9]+(?:-[a-z0-9]+)*$")
by_name = {}
for suite in suites:
    name = suite.get("name", "")
    if not name_re.match(name):
        errors.append(f"invalid suite name: {name}")
    if name in seen:
        errors.append(f"duplicate suite name: {name}")
    seen.add(name)
    by_name[name] = suite

    for req in ("purpose", "kind", "expected_metrics", "thresholds", "must_pass"):
        if req not in suite:
            errors.append(f"{name}: missing required field '{req}'")

    if suite.get("kind") == "k6":
        scenario = suite.get("scenario")
        if not scenario:
            errors.append(f"{name}: kind=k6 requires scenario")
        else:
            scenario_path = ROOT / manifest["scenarios_dir"] / scenario
            if not scenario_path.exists():
                errors.append(f"{name}: scenario file missing: {scenario_path}")
    elif suite.get("kind") == "script":
        script = suite.get("script")
        if not script:
            errors.append(f"{name}: kind=script requires script")
        else:
            script_path = ROOT / script
            if not script_path.exists():
                errors.append(f"{name}: script missing: {script_path}")

    thresholds = suite.get("thresholds", {})
    if not any(
        k in thresholds
        for k in (
            "p95_ms_max",
            "p99_ms_max",
            "error_rate_max",
            "cold_start_p99_ms_max",
            "max_pod_cold_start_ms_max",
            "memory_growth_bytes_max",
        )
    ):
        errors.append(f"{name}: thresholds must define at least one limit")

    expected_metrics = suite.get("expected_metrics", [])
    if not isinstance(expected_metrics, list) or not expected_metrics:
        errors.append(f"{name}: expected_metrics must be non-empty")

for required in ("cheap-only-survival", "store-outage-under-spike", "pod-churn"):
    suite = by_name.get(required)
    if not suite:
        errors.append(f"missing required suite: {required}")
        continue
    if not suite.get("must_pass", False):
        errors.append(f"{required}: must_pass must be true")

soak = by_name.get("soak-30m")
if not soak:
    errors.append("missing required suite: soak-30m")
else:
    run_in = set(soak.get("run_in", []))
    if "nightly" not in run_in and "load-nightly" not in run_in:
        errors.append("soak-30m: must be in nightly profile")
    if "smoke" in run_in or "pr" in run_in or "load-ci" in run_in:
        errors.append("soak-30m: must not run in smoke/pr/load-ci profiles")

redis_suite = by_name.get("redis-optional")
if redis_suite and not redis_suite.get("requires", {}).get("redis_experiment", False):
    errors.append("redis-optional: requires.redis_experiment must be true")

from atlasctl.checks.domains.ops.ops_checks import check_ops_load_pinned_queries_lock_native

lock_code, lock_errors = check_ops_load_pinned_queries_lock_native(ROOT)
if lock_code != 0:
    errors.extend(lock_errors)

scenarios_dir = ROOT / manifest["scenarios_dir"]
registered = {
    s["scenario"] for s in suites if s.get("kind") == "k6" and isinstance(s.get("scenario"), str) and s["scenario"]
}
for file in sorted(scenarios_dir.glob("*.js")):
    if file.name not in registered:
        errors.append(f"unregistered k6 scenario file: {file.relative_to(ROOT)}")

if errors:
    for err in errors:
        print(err, file=sys.stderr)
    sys.exit(1)

print("suite manifest validation passed")
