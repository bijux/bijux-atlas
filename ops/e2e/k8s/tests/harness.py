#!/usr/bin/env python3
import argparse
import json
import os
import subprocess
import sys
import time
from pathlib import Path
from xml.sax.saxutils import escape


def load_manifest(path: Path):
    with path.open() as f:
        data = json.load(f)
    tests = data.get("tests", [])
    known = {t.get("script") for t in tests}
    test_dir = path.parent
    for extra in sorted(test_dir.glob("test_*.sh")):
        if extra.name in known:
            continue
        tests.append(
            {
                "script": extra.name,
                "groups": ["legacy"],
                "retries": 1,
                "expected_failure_modes": ["unspecified"],
            }
        )
    return tests


def select_tests(tests, groups, names):
    out = []
    for t in tests:
        tg = set(t.get("groups", []))
        if groups and tg.isdisjoint(groups):
            continue
        if names and t["script"] not in names:
            continue
        out.append(t)
    return out


def run_one(script_path: Path, retries: int, env):
    attempts = []
    for attempt in range(1, retries + 1):
        start = time.time()
        proc = subprocess.run([str(script_path)], text=True, capture_output=True, env=env)
        duration = time.time() - start
        attempt_result = {
            "attempt": attempt,
            "exit_code": proc.returncode,
            "duration_seconds": round(duration, 3),
            "stdout": proc.stdout,
            "stderr": proc.stderr,
        }
        attempts.append(attempt_result)
        if proc.returncode == 0:
            break
    return attempts


def write_junit(results, out_path: Path):
    failures = sum(1 for r in results if r["status"] != "passed")
    total_time = sum(r["duration_seconds"] for r in results)
    lines = [
        '<?xml version="1.0" encoding="UTF-8"?>',
        f'<testsuite name="ops-e2e-k8s" tests="{len(results)}" failures="{failures}" errors="0" skipped="0" time="{total_time:.3f}">',
    ]
    for r in results:
        lines.append(f'  <testcase classname="ops.e2e.k8s" name="{escape(r["script"])}" time="{r["duration_seconds"]:.3f}">')
        if r["status"] != "passed":
            last = r["attempts"][-1]
            msg = escape((last.get("stderr") or "").strip()[:2000])
            lines.append(f'    <failure message="test failed">{msg}</failure>')
        lines.append("  </testcase>")
    lines.append("</testsuite>")
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text("\n".join(lines) + "\n")


def main():
    parser = argparse.ArgumentParser(description="Run ops/e2e/k8s tests with retries and reports")
    parser.add_argument("--group", action="append", default=[])
    parser.add_argument("--test", action="append", default=[])
    parser.add_argument("--manifest", default="ops/e2e/k8s/tests/manifest.json")
    parser.add_argument("--retries", type=int, default=1)
    parser.add_argument("--json-out", default="artifacts/ops/k8s/test-results.json")
    parser.add_argument("--junit-out", default="artifacts/ops/k8s/test-results.xml")
    args = parser.parse_args()

    repo_root = Path(__file__).resolve().parents[4]
    os.chdir(repo_root)
    run_id = os.environ.get("ATLAS_RUN_ID", "local")
    os.environ.setdefault("ATLAS_E2E_NAMESPACE", f"atlas-e2e-{run_id}")

    tests = load_manifest(Path(args.manifest))
    selected = select_tests(tests, set(args.group), set(args.test))
    if not selected:
        print("no tests selected", file=sys.stderr)
        return 2

    results = []
    failed = 0
    for t in selected:
        script = t["script"]
        retries = max(args.retries, int(t.get("retries", 1)))
        spath = repo_root / "ops/e2e/k8s/tests" / script
        if not spath.exists():
            results.append({"script": script, "status": "failed", "duration_seconds": 0.0, "attempts": [{"attempt": 1, "exit_code": 127, "stdout": "", "stderr": "script missing", "duration_seconds": 0.0}], "groups": t.get("groups", []), "expected_failure_modes": t.get("expected_failure_modes", [])})
            failed += 1
            continue

        attempts = run_one(spath, retries, os.environ.copy())
        status = "passed" if attempts[-1]["exit_code"] == 0 else "failed"
        duration = sum(a["duration_seconds"] for a in attempts)
        results.append({
            "script": script,
            "status": status,
            "duration_seconds": round(duration, 3),
            "attempts": attempts,
            "groups": t.get("groups", []),
            "expected_failure_modes": t.get("expected_failure_modes", []),
        })
        if status != "passed":
            failed += 1

    payload = {
        "run_id": run_id,
        "namespace": os.environ.get("ATLAS_E2E_NAMESPACE"),
        "timestamp": int(time.time()),
        "total": len(results),
        "failed": failed,
        "passed": len(results) - failed,
        "results": results,
    }

    jout = Path(args.json_out)
    jout.parent.mkdir(parents=True, exist_ok=True)
    jout.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n")
    write_junit(results, Path(args.junit_out))

    print(f"k8s test harness: {len(results)} tests, failed={failed}")
    print(f"json: {args.json_out}")
    print(f"junit: {args.junit_out}")
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
