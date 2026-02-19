#!/usr/bin/env python3
from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
TESTS_DIR = ROOT / "ops/k8s/tests"
MANIFEST_PATH = TESTS_DIR / "manifest.json"
SUITES_PATH = TESTS_DIR / "suites.json"


def main() -> int:
    manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
    suites = json.loads(SUITES_PATH.read_text(encoding="utf-8"))
    errors: list[str] = []

    manifest_tests = manifest.get("tests", [])
    manifest_scripts = {
        str(t.get("script", "")).strip()
        for t in manifest_tests
        if isinstance(t, dict) and str(t.get("script", "")).strip()
    }
    manifest_groups = {
        g
        for t in manifest_tests
        if isinstance(t, dict)
        for g in t.get("groups", [])
        if isinstance(g, str) and g
    }

    disk_test_scripts = {
        p.name
        for p in TESTS_DIR.glob("test_*.sh")
        if p.is_file()
    }

    missing_in_manifest = sorted(disk_test_scripts - manifest_scripts)
    for script in missing_in_manifest:
        errors.append(f"test script exists but is not declared in manifest.json: {script}")

    unknown_in_manifest = sorted(s for s in manifest_scripts if "/" not in s and s.startswith("test_") and s not in disk_test_scripts)
    for script in unknown_in_manifest:
        errors.append(f"manifest references missing test script: {script}")

    suite_ids: set[str] = set()
    suite_groups: set[str] = set()
    for suite in suites.get("suites", []):
        sid = suite.get("id")
        if not isinstance(sid, str) or not sid:
            errors.append("suites.json contains suite without valid id")
            continue
        if sid in suite_ids:
            errors.append(f"duplicate k8s suite id: {sid}")
        suite_ids.add(sid)

        groups = suite.get("groups", [])
        if not isinstance(groups, list):
            errors.append(f"suite `{sid}` groups must be a list")
            continue
        for g in groups:
            if not isinstance(g, str) or not g:
                errors.append(f"suite `{sid}` has invalid group entry")
                continue
            suite_groups.add(g)
            if g not in manifest_groups:
                errors.append(f"suite `{sid}` references unknown group `{g}` (missing in manifest.json)")

    full_suite = next((s for s in suites.get("suites", []) if s.get("id") == "full"), None)
    full_is_wildcard = isinstance(full_suite, dict) and full_suite.get("groups", []) == []
    if not full_is_wildcard:
        uncovered_groups = sorted(manifest_groups - suite_groups)
        for g in uncovered_groups:
            errors.append(f"manifest group `{g}` is not reachable by any suite in suites.json")

    if "full" not in suite_ids:
        errors.append("suites.json must include suite id `full`")

    if errors:
        print("k8s suites authoritative check failed:", file=sys.stderr)
        for err in errors:
            print(f"- {err}", file=sys.stderr)
        return 1

    print("k8s suites authoritative check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
