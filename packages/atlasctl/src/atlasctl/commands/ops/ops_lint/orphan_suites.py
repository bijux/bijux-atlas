#!/usr/bin/env python3
from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]
errors: list[str] = []


def check_load() -> None:
    manifest = json.loads((ROOT / "ops/load/suites/suites.json").read_text(encoding="utf-8"))
    scenarios_dir = ROOT / manifest["scenarios_dir"]
    scenario_files = {p.name for p in scenarios_dir.glob("*.json")}
    manifest_scenarios = {
        s.get("scenario") for s in manifest.get("suites", []) if s.get("kind") == "k6"
    }
    missing = sorted(s for s in manifest_scenarios if s and s not in scenario_files)
    for s in missing:
        errors.append(f"load suite references missing scenario: {s}")

    referenced_js: set[str] = set()
    for scenario in scenarios_dir.glob("*.json"):
        payload = json.loads(scenario.read_text(encoding="utf-8"))
        suite = payload.get("suite")
        if isinstance(suite, str) and suite.endswith(".js"):
            referenced_js.add(suite)

    for js in sorted((ROOT / "ops/load/k6/suites").glob("*.js")):
        if js.name not in referenced_js:
            errors.append(f"orphan load k6 suite: {js.relative_to(ROOT).as_posix()}")


def check_k8s() -> None:
    suites = json.loads((ROOT / "ops/k8s/tests/suites.json").read_text(encoding="utf-8"))
    manifest = json.loads((ROOT / "ops/k8s/tests/manifest.json").read_text(encoding="utf-8"))
    manifest_groups = {
        g
        for test in manifest.get("tests", [])
        if isinstance(test, dict)
        for g in test.get("groups", [])
        if isinstance(g, str) and g
    }
    if not any(s.get("id") == "full" for s in suites.get("suites", [])):
        errors.append("k8s suites.json must include `full`")
    for suite in suites.get("suites", []):
        suite_id = suite.get("id")
        if not isinstance(suite_id, str) or not suite_id:
            errors.append("k8s suites.json contains suite with invalid id")
            continue
        groups = suite.get("groups", [])
        if not isinstance(groups, list):
            errors.append(f"k8s suite `{suite_id}` has non-list groups")
            continue
        for group in groups:
            if group not in manifest_groups:
                errors.append(f"k8s suite `{suite_id}` references unknown group `{group}`")


def check_obs() -> None:
    suites = json.loads((ROOT / "ops/obs/tests/suites.json").read_text(encoding="utf-8"))
    test_files = {p.name for p in (ROOT / "ops/obs/tests").glob("test_*.sh")}
    if not any(s.get("id") == "full" for s in suites.get("suites", [])):
        errors.append("obs suites.json must include `full`")
    for suite in suites.get("suites", []):
        suite_id = suite.get("id")
        tests = suite.get("tests", [])
        if not isinstance(suite_id, str) or not suite_id:
            errors.append("obs suites.json contains suite with invalid id")
            continue
        if not isinstance(tests, list):
            errors.append(f"obs suite `{suite_id}` has non-list tests")
            continue
        for test in tests:
            if test not in test_files:
                errors.append(f"obs suite `{suite_id}` references missing test `{test}`")


def check_e2e() -> None:
    scenarios = json.loads((ROOT / "ops/e2e/scenarios/scenarios.json").read_text(encoding="utf-8"))
    if not scenarios.get("scenarios"):
        errors.append("ops/e2e/scenarios/scenarios.json must declare scenarios")

    realdata = json.loads((ROOT / "ops/e2e/realdata/scenarios.json").read_text(encoding="utf-8"))
    for scenario in realdata.get("scenarios", []):
        script = scenario.get("script")
        scenario_id = scenario.get("id")
        if not isinstance(script, str) or not script:
            errors.append(f"realdata scenario `{scenario_id}` missing script")
            continue
        if not (ROOT / "ops/e2e/realdata" / script).is_file():
            errors.append(f"realdata scenario `{scenario_id}` references missing script `{script}`")


check_load()
check_k8s()
check_obs()
check_e2e()

if errors:
    for e in errors:
        print(e, file=sys.stderr)
    raise SystemExit(1)

print("all k8s/load/obs/e2e suites are referenced")
