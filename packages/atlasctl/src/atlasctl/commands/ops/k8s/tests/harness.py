#!/usr/bin/env python3
import argparse
import datetime as dt
import json
import os
import re
import subprocess
import sys
import time
from pathlib import Path
from signal import SIGKILL, SIGTERM
from xml.sax.saxutils import escape


def today_utc() -> dt.date:
    return dt.datetime.now(dt.timezone.utc).date()


def parse_date(s: str) -> dt.date:
    return dt.datetime.strptime(s, "%Y-%m-%d").date()


def load_manifest(path: Path):
    with path.open() as f:
        return json.load(f)


def _extract_emitted_failure_modes(text: str) -> list[str]:
    modes: list[str] = []
    for m in re.findall(r"failure_mode\s*[:=]\s*([a-z0-9_]+)", text, flags=re.IGNORECASE):
        modes.append(m.lower())
    return modes


def _classify_observed_failure_mode(attempts, expected_modes):
    expected = {m.lower() for m in expected_modes if isinstance(m, str)}
    emitted: list[str] = []
    for a in attempts:
        emitted.extend(_extract_emitted_failure_modes((a.get("stderr") or "") + "\n" + (a.get("stdout") or "")))
    undeclared = sorted({m for m in emitted if m not in expected})
    if undeclared:
        return "undeclared_failure_mode", sorted(set(emitted)), undeclared
    for m in emitted:
        if m in expected:
            return m, sorted(set(emitted)), []
    return "unclassified", sorted(set(emitted)), []


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


def run_one(script_path: Path, retries: int, timeout_seconds: int, env, test_artifact_dir: Path):
    attempts = []
    for attempt in range(1, retries + 1):
        start = time.time()
        attempt_dir = test_artifact_dir / f"attempt-{attempt}"
        attempt_dir.mkdir(parents=True, exist_ok=True)
        env_with_attempt = env.copy()
        env_with_attempt["ATLAS_TEST_ARTIFACT_DIR"] = str(attempt_dir)
        try:
            proc = subprocess.Popen(
                [str(script_path)],
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                env=env_with_attempt,
                preexec_fn=os.setsid if os.name != "nt" else None,
            )
            try:
                stdout, stderr = proc.communicate(timeout=timeout_seconds)
            except subprocess.TimeoutExpired:
                if os.name != "nt":
                    os.killpg(proc.pid, SIGTERM)
                    time.sleep(0.5)
                    try:
                        os.killpg(proc.pid, SIGKILL)
                    except ProcessLookupError:
                        pass
                else:
                    proc.kill()
                stdout, stderr = proc.communicate()
                raise subprocess.TimeoutExpired(proc.args, timeout_seconds, output=stdout, stderr=stderr)
            duration = time.time() - start
            attempt_result = {
                "attempt": attempt,
                "exit_code": proc.returncode,
                "duration_seconds": round(duration, 3),
                "timed_out": False,
                "stdout": stdout,
                "stderr": stderr,
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
        (attempt_dir / "stdout.txt").write_text(attempt_result["stdout"], encoding="utf-8")
        (attempt_dir / "stderr.txt").write_text(attempt_result["stderr"], encoding="utf-8")
        (attempt_dir / "attempt.json").write_text(json.dumps(attempt_result, indent=2, sort_keys=True) + "\n", encoding="utf-8")
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


def maybe_collect_failure_bundle(repo_root: Path, failed: bool, env, out_dir: Path):
    if not failed:
        return
    out_dir.mkdir(parents=True, exist_ok=True)
    cmd = [
        "python3",
        str(repo_root / "packages/atlasctl/src/atlasctl/commands/ops/k8s/tests/failure_report.py"),
        env.get("ATLAS_E2E_NAMESPACE", "atlas-e2e-local"),
        env.get("ATLAS_E2E_RELEASE_NAME", "atlas-e2e"),
    ]
    proc = subprocess.run(cmd, env=env, text=True, capture_output=True)
    (out_dir / "k8s-test-report.stdout.txt").write_text(proc.stdout or "", encoding="utf-8")
    (out_dir / "k8s-test-report.stderr.txt").write_text(proc.stderr or "", encoding="utf-8")


def _validate_report_payload(payload):
    errors = []
    if payload.get("schema_version") != 1:
        errors.append("schema_version must be 1")
    for req in ("run_id", "suite_id", "total", "failed", "passed", "results"):
        if req not in payload:
            errors.append(f"missing required report key `{req}`")
    if not isinstance(payload.get("results"), list):
        errors.append("results must be a list")
    return errors


def main():
    parser = argparse.ArgumentParser(description="Run ops/e2e/k8s tests with retries and reports")
    parser.add_argument("--group", action="append", default=[])
    parser.add_argument("--test", action="append", default=[])
    parser.add_argument("--manifest", default="ops/k8s/tests/manifest.json")
    parser.add_argument("--retries", type=int, default=1)
    parser.add_argument("--json-out", default="artifacts/ops/k8s/test-results.json")
    parser.add_argument("--junit-out", default="artifacts/ops/k8s/test-results.xml")
    parser.add_argument("--suite-id", default="adhoc")
    parser.add_argument("--fail-fast", action="store_true")
    parser.add_argument("--include-quarantined", action="store_true")
    args = parser.parse_args()

    cur = Path(__file__).resolve()
    repo_root = None
    for parent in cur.parents:
        if all((parent / marker).exists() for marker in ("makefiles", "packages", "configs", "ops")):
            repo_root = parent
            break
    if repo_root is None:
        raise RuntimeError("unable to resolve repo root")
    os.chdir(repo_root)
    run_id = os.environ.get("ATLAS_RUN_ID", "local")
    os.environ.setdefault("ATLAS_E2E_NAMESPACE", f"atlas-e2e-{run_id}")

    manifest_doc = load_manifest(Path(args.manifest))
    tests = manifest_doc.get("tests", [])
    flake_policy = manifest_doc.get("flake_policy", {})
    selected = select_tests(tests, set(args.group), set(args.test))
    selected.sort(key=lambda t: t["script"])
    if not selected:
        print("no tests selected", file=sys.stderr)
        return 2

    suite_start = time.time()
    per_test_root = Path(args.json_out).parent / "tests"
    results = []
    failed = 0
    flaky = []
    fail_fast_triggered = False
    for t in selected:
        script = t["script"]
        quarantine_until = t.get("quarantine_until")
        if quarantine_until and not args.include_quarantined and today_utc() <= parse_date(quarantine_until):
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

        test_artifact_dir = per_test_root / Path(script).stem
        attempts = run_one(spath, retries, timeout_seconds, os.environ.copy(), test_artifact_dir)
        status = "passed" if attempts[-1]["exit_code"] == 0 else "failed"
        duration = sum(a["duration_seconds"] for a in attempts)
        if status == "passed" and len(attempts) > 1:
            flaky.append({"script": script, "attempts": len(attempts), "owner": t.get("owner", "unknown")})
        observed_mode = None
        emitted_modes = []
        undeclared_modes = []
        if status != "passed":
            observed_mode, emitted_modes, undeclared_modes = _classify_observed_failure_mode(attempts, t.get("expected_failure_modes", []))

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
                "observed_failure_mode": observed_mode,
                "emitted_failure_modes": emitted_modes,
                "undeclared_failure_modes": undeclared_modes,
                "artifacts_dir": str(test_artifact_dir),
            }
        )
        if status != "passed":
            failed += 1
            if args.fail_fast:
                fail_fast_triggered = True
                break
        print(f"[{len(results)}/{len(selected)}] {script}: {status} ({duration:.2f}s)")

    total_duration = round(time.time() - suite_start, 3)
    payload = {
        "schema_version": 1,
        "run_id": run_id,
        "suite_id": args.suite_id,
        "namespace": os.environ.get("ATLAS_E2E_NAMESPACE"),
        "timestamp": int(time.time()),
        "duration_seconds": total_duration,
        "total": len(results),
        "failed": failed,
        "passed": sum(1 for r in results if r["status"] == "passed"),
        "skipped": sum(1 for r in results if r["status"] == "skipped"),
        "flake_count": len(flaky),
        "flakes": flaky,
        "fail_fast": bool(args.fail_fast),
        "fail_fast_triggered": fail_fast_triggered,
        "quarantine_enforced": not args.include_quarantined,
        "flake_policy": flake_policy,
        "results": results,
    }
    report_errors = _validate_report_payload(payload)
    if report_errors:
        for err in report_errors:
            print(f"suite report validation error: {err}", file=sys.stderr)
        return 2

    jout = Path(args.json_out)
    jout.parent.mkdir(parents=True, exist_ok=True)
    jout.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n")
    write_junit(results, Path(args.junit_out))

    flake_out = Path("artifacts/ops/k8s/flake-report.json")
    flake_out.parent.mkdir(parents=True, exist_ok=True)
    flake_out.write_text(json.dumps({"flake_count": len(flaky), "flakes": flaky}, indent=2, sort_keys=True) + "\n")

    maybe_collect_failure_bundle(repo_root, failed > 0, os.environ.copy(), jout.parent / "failure-bundle")

    print(f"k8s test harness: {len(results)} tests, failed={failed}, flakes={len(flaky)}")
    print(f"json: {args.json_out}")
    print(f"junit: {args.junit_out}")
    if flaky:
        print("flake policy: retry-pass tests detected; quarantine with TTL and open issue required", file=sys.stderr)
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
