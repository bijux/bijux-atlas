#!/usr/bin/env python3
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
manifest_path = ROOT / "ops/load/suites/suites.json"
schema_path = ROOT / "ops/load/contracts/suite-schema.json"
manifest = json.loads(manifest_path.read_text())
schema = json.loads(schema_path.read_text())

errors = []

for key in schema.get("required", []):
    if key not in manifest:
        errors.append(f"missing required root key: {key}")

suites = manifest.get("suites", [])
seen = set()
name_re = re.compile(r"^[a-z0-9]+(?:-[a-z0-9]+)*$")
by_name = {}
for s in suites:
    name = s.get("name", "")
    if not name_re.match(name):
        errors.append(f"invalid suite name: {name}")
    if name in seen:
        errors.append(f"duplicate suite name: {name}")
    seen.add(name)
    by_name[name] = s

    for req in ("purpose", "kind", "expected_metrics", "thresholds", "must_pass"):
        if req not in s:
            errors.append(f"{name}: missing required field '{req}'")

    if s.get("kind") == "k6":
        scenario = s.get("scenario")
        if not scenario:
            errors.append(f"{name}: kind=k6 requires scenario")
        else:
            p = ROOT / manifest["scenarios_dir"] / scenario
            if not p.exists():
                errors.append(f"{name}: scenario file missing: {p}")
    elif s.get("kind") == "script":
        script = s.get("script")
        if not script:
            errors.append(f"{name}: kind=script requires script")
        else:
            p = ROOT / script
            if not p.exists():
                errors.append(f"{name}: script missing: {p}")

    t = s.get("thresholds", {})
    if not any(k in t for k in ("p95_ms_max", "p99_ms_max", "error_rate_max", "cold_start_p99_ms_max", "max_pod_cold_start_ms_max", "memory_growth_bytes_max")):
        errors.append(f"{name}: thresholds must define at least one limit")

    em = s.get("expected_metrics", [])
    if not isinstance(em, list) or not em:
        errors.append(f"{name}: expected_metrics must be non-empty")

# Hard policy checks for required suites.
for required in ("cheap-only-survival", "store-outage-mid-spike", "pod-churn"):
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
if redis_suite:
    if not redis_suite.get("requires", {}).get("redis_experiment", False):
        errors.append("redis-optional: requires.redis_experiment must be true")

# Verify query lock every time manifest is validated.
lock_script = ROOT / "ops/load/scripts/check_pinned_queries_lock.py"
if lock_script.exists():
    import subprocess

    proc = subprocess.run([str(lock_script)], capture_output=True, text=True)
    if proc.returncode != 0:
        errors.append(proc.stderr.strip() or proc.stdout.strip() or "pinned query lock check failed")

if errors:
    for e in errors:
        print(e, file=sys.stderr)
    sys.exit(1)

print("suite manifest validation passed")
