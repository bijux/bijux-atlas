#!/usr/bin/env python3
from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path


def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for parent in cur.parents:
        if all((parent / marker).exists() for marker in ("makefiles", "packages", "configs", "ops")):
            return parent
    raise RuntimeError("unable to resolve repo root")


def _suite_groups_and_fail_fast(suites_path: Path, suite_id: str) -> tuple[list[str], bool]:
    payload = json.loads(suites_path.read_text(encoding="utf-8"))
    for suite in payload.get("suites", []):
        if suite.get("id") == suite_id:
            groups = sorted([g for g in suite.get("groups", []) if isinstance(g, str) and g])
            return groups, bool(suite.get("fail_fast", False))
    raise SystemExit(f"unknown suite id: {suite_id}")


def main() -> int:
    root = _repo_root()
    tests_dir = root / "ops/k8s/tests"

    env = os.environ
    json_out = env.get("ATLAS_E2E_JSON_OUT", "ops/_artifacts/k8s/test-results.json")
    junit_out = env.get("ATLAS_E2E_JUNIT_OUT", "ops/_artifacts/k8s/test-results.xml")
    summary_out = env.get("ATLAS_E2E_SUMMARY_OUT", "ops/_artifacts/k8s/test-summary.md")
    degradation_score_out = env.get(
        "ATLAS_E2E_DEGRADATION_SCORE_OUT", f"{Path(json_out).parent}/graceful-degradation-score.json"
    )
    conformance_out = env.get(
        "ATLAS_E2E_CONFORMANCE_OUT", f"{Path(json_out).parent}/k8s-conformance-report.json"
    )
    retries = env.get("ATLAS_E2E_RETRIES", "1")
    fail_fast = env.get("ATLAS_E2E_FAIL_FAST", "0") == "1"
    include_quarantined = env.get("ATLAS_E2E_INCLUDE_QUARANTINED", "0") == "1"

    argv = list(sys.argv[1:])
    suite = ""
    if len(argv) >= 2 and argv[0] == "--suite":
        suite = argv[1]
        argv = argv[2:]

    suite_id = suite or "adhoc"
    group_args: list[str] = []
    if suite:
        groups, suite_fail_fast = _suite_groups_and_fail_fast(tests_dir / "suites.json", suite)
        if suite_fail_fast:
            fail_fast = True
        for grp in groups:
            group_args.extend(["--group", grp])

    subprocess.run(
        [
            "python3",
            str(root / "packages/atlasctl/src/atlasctl/observability/contracts/governance/check_tool_versions.py"),
            "kind",
            "kubectl",
            "helm",
        ],
        check=True,
        cwd=str(root),
        stdout=subprocess.DEVNULL,
    )

    harness_args = [
        "python3",
        str(root / "packages/atlasctl/src/atlasctl/commands/ops/k8s/tests/harness.py"),
        "--manifest",
        str(tests_dir / "manifest.json"),
        "--json-out",
        json_out,
        "--junit-out",
        junit_out,
        "--retries",
        str(retries),
        "--suite-id",
        suite_id,
    ]
    if fail_fast:
        harness_args.append("--fail-fast")
    if include_quarantined:
        harness_args.append("--include-quarantined")
    harness_args.extend(group_args)
    harness_args.extend(argv)

    status = subprocess.run(harness_args, cwd=str(root), check=False).returncode
    post_steps = [
        [
            "python3",
            str(root / "packages/atlasctl/src/atlasctl/commands/ops/k8s/tests/validate_report.py"),
            "--report",
            json_out,
        ],
        [
            "python3",
            str(root / "packages/atlasctl/src/atlasctl/commands/ops/k8s/tests/render_summary.py"),
            "--json",
            json_out,
            "--out",
            summary_out,
        ],
        [
            "python3",
            str(root / "packages/atlasctl/src/atlasctl/commands/ops/k8s/tests/compute_graceful_degradation_score.py"),
            "--json",
            json_out,
            "--out",
            degradation_score_out,
        ],
        [
            "python3",
            str(root / "packages/atlasctl/src/atlasctl/commands/ops/k8s/tests/build_conformance_report.py"),
            "--json",
            json_out,
            "--out",
            conformance_out,
        ],
    ]
    for cmd in post_steps:
        subprocess.run(cmd, cwd=str(root), check=True)
    return status


if __name__ == "__main__":
    raise SystemExit(main())
