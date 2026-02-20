#!/usr/bin/env python3
import argparse
import datetime as dt
import json
import os
import subprocess
import sys
import time
from pathlib import Path
from xml.sax.saxutils import escape


def today_utc() -> dt.date:
    return dt.datetime.now(dt.timezone.utc).date()


def parse_date(s: str) -> dt.date:
    return dt.datetime.strptime(s, "%Y-%m-%d").date()


def load_manifest(path: Path):
    with path.open() as f:
        data = json.load(f)
    return data.get("tests", [])


def select_tests(tests, groups, names):
    out = []
    for t in tests:
        tg = set(t.get("groups", []))
        if groups and tg.isdisjoint(groups):
            continue
        if names and t["script"] not in names and Path(t["script"]).name not in names:
            continue
        out.append(t)
    return out


def run_one(script_path: Path, retries: int, timeout_seconds: int, env):
    attempts = []
    for attempt in range(1, retries + 1):
        start = time.time()
        try:
            proc = subprocess.run(
                [str(script_path)],
                text=True,
                capture_output=True,
                env=env,
                timeout=timeout_seconds,
            )
            duration = time.time() - start
            attempt_result = {
                "attempt": attempt,
                "exit_code": proc.returncode,
                "duration_seconds": round(duration, 3),
                "timed_out": False,
                "stdout": proc.stdout,
                "stderr": proc.stderr,
            }
        except subprocess.TimeoutExpired as exc:
            duration = time.time() - start
            attempt_result = {
                "attempt": attempt,
                "exit_code": 124,
                "duration_seconds": round(duration, 3),
                "timed_out": True,
                "stdout": exc.stdout or "",
                "stderr": (exc.stderr or "") + f"\nTimed out after {timeout_seconds}s",
            }
        attempts.append(attempt_result)
        if attempt_result["exit_code"] == 0:
            break
    return attempts


def write_junit(results, out_path: Path):
    failures = sum(1 for r in results if r["status"] == "failed")
    skipped = sum(1 for r in results if r["status"] == "skipped")
    total_time = sum(r["duration_seconds"] for r in results)
    lines = [
        '<?xml version="1.0" encoding="UTF-8"?>',
        f'<testsuite name="ops-e2e-k8s" tests="{len(results)}" failures="{failures}" errors="0" skipped="{skipped}" time="{total_time:.3f}">',
    ]
    for r in results:
        lines.append(f'  <testcase classname="ops.e2e.k8s" name="{escape(r["script"])}" time="{r["duration_seconds"]:.3f}">')
        if r["status"] == "failed":
            last = r["attempts"][-1]
            msg = escape((last.get("stderr") or "").strip()[:2000])
            lines.append(f'    <failure message="test failed">{msg}</failure>')
        elif r["status"] == "skipped":
            reason = escape(r.get("skip_reason", "quarantined"))
            lines.append(f'    <skipped message="{reason}"/>')
        lines.append("  </testcase>")
    lines.append("</testsuite>")
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text("\n".join(lines) + "\n")


def maybe_collect_failure_bundle(repo_root: Path, failed: bool, env):
    if not failed:
        return
    cmd = [str(repo_root / "ops/_lib/k8s-test-report.sh"), env.get("ATLAS_E2E_NAMESPACE", "atlas-e2e-local"), env.get("ATLAS_E2E_RELEASE_NAME", "atlas-e2e")]
    subprocess.run(cmd, env=env, text=True, capture_output=True)


def main():
    parser = argparse.ArgumentParser(description="Run ops/e2e/k8s tests with retries and reports")
    parser.add_argument("--group", action="append", default=[])
    parser.add_argument("--test", action="append", default=[])
    parser.add_argument("--manifest", default="ops/k8s/tests/manifest.json")
    parser.add_argument("--retries", type=int, default=1)
    parser.add_argument("--json-out", default="artifacts/ops/k8s/test-results.json")
    parser.add_argument("--junit-out", default="artifacts/ops/k8s/test-results.xml")
    args = parser.parse_args()

    repo_root = Path(__file__).resolve().parents[3]
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
    flaky = []
    for t in selected:
        script = t["script"]
        quarantine_until = t.get("quarantine_until")
        if quarantine_until and today_utc() <= parse_date(quarantine_until):
            results.append(
                {
                    "script": script,
                    "status": "skipped",
                    "skip_reason": f"quarantined until {quarantine_until}",
                    "duration_seconds": 0.0,
                    "attempts": [],
                    "groups": t.get("groups", []),
                    "owner": t.get("owner", "unknown"),
                    "expected_failure_modes": t.get("expected_failure_modes", []),
                }
            )
            continue

        retries = max(args.retries, int(t.get("retries", 1)))
        timeout_seconds = int(t.get("timeout_seconds", 600))
        if "flake-sensitive" in t.get("groups", []) and retries > 1 and not t.get("flake_issue_id"):
            results.append(
                {
                    "script": script,
                    "status": "failed",
                    "duration_seconds": 0.0,
                    "attempts": [
                        {
                            "attempt": 1,
                            "exit_code": 2,
                            "stdout": "",
                            "stderr": "flake-sensitive test retries>1 requires flake_issue_id",
                            "duration_seconds": 0.0,
                            "timed_out": False,
                        }
                    ],
                    "groups": t.get("groups", []),
                    "owner": t.get("owner", "unknown"),
                    "timeout_seconds": timeout_seconds,
                    "expected_failure_modes": t.get("expected_failure_modes", []),
                }
            )
            failed += 1
            continue
        spath = repo_root / "ops/k8s/tests" / script
        if not spath.exists():
            results.append(
                {
                    "script": script,
                    "status": "failed",
                    "duration_seconds": 0.0,
                    "attempts": [{"attempt": 1, "exit_code": 127, "stdout": "", "stderr": "script missing", "duration_seconds": 0.0, "timed_out": False}],
                    "groups": t.get("groups", []),
                    "owner": t.get("owner", "unknown"),
                    "expected_failure_modes": t.get("expected_failure_modes", []),
                }
            )
            failed += 1
            continue

        attempts = run_one(spath, retries, timeout_seconds, os.environ.copy())
        status = "passed" if attempts[-1]["exit_code"] == 0 else "failed"
        duration = sum(a["duration_seconds"] for a in attempts)
        if status == "passed" and len(attempts) > 1:
            flaky.append({"script": script, "attempts": len(attempts), "owner": t.get("owner", "unknown")})

        results.append(
            {
                "script": script,
                "status": status,
                "duration_seconds": round(duration, 3),
                "attempts": attempts,
                "groups": t.get("groups", []),
                "owner": t.get("owner", "unknown"),
                "timeout_seconds": timeout_seconds,
                "expected_failure_modes": t.get("expected_failure_modes", []),
            }
        )
        if status != "passed":
            failed += 1

    payload = {
        "run_id": run_id,
        "namespace": os.environ.get("ATLAS_E2E_NAMESPACE"),
        "timestamp": int(time.time()),
        "total": len(results),
        "failed": failed,
        "passed": sum(1 for r in results if r["status"] == "passed"),
        "skipped": sum(1 for r in results if r["status"] == "skipped"),
        "flake_count": len(flaky),
        "flakes": flaky,
        "results": results,
    }

    jout = Path(args.json_out)
    jout.parent.mkdir(parents=True, exist_ok=True)
    jout.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n")
    write_junit(results, Path(args.junit_out))

    flake_out = Path("artifacts/ops/k8s/flake-report.json")
    flake_out.parent.mkdir(parents=True, exist_ok=True)
    flake_out.write_text(json.dumps({"flake_count": len(flaky), "flakes": flaky}, indent=2, sort_keys=True) + "\n")

    maybe_collect_failure_bundle(repo_root, failed > 0, os.environ.copy())

    print(f"k8s test harness: {len(results)} tests, failed={failed}, flakes={len(flaky)}")
    print(f"json: {args.json_out}")
    print(f"junit: {args.junit_out}")
    if flaky:
        print("flake policy: retry-pass tests detected; quarantine with TTL and open issue required", file=sys.stderr)
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
