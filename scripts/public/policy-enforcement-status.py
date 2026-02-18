#!/usr/bin/env python3
# owner: platform
# purpose: validate policy enforcement coverage table and generate status doc.
# stability: public
# called-by: make policy-enforcement-status, make ci-policy-enforcement
from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
COVERAGE = ROOT / "configs/policy/policy-enforcement-coverage.json"
OUT = ROOT / "docs/_generated/policy-enforcement-status.md"


def rg_exists(pattern: str) -> bool:
    cmd = ["rg", "-n", "--fixed-strings", pattern, "crates", "scripts", "makefiles", "docs"]
    return subprocess.run(cmd, cwd=ROOT, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL).returncode == 0


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--enforce", action="store_true")
    args = parser.parse_args()

    data = json.loads(COVERAGE.read_text())
    hard = set(data.get("hard_policies", []))
    rows = []
    violations: list[str] = []
    covered_hard = 0
    total_hard = len(hard)

    for policy in data.get("policies", []):
        pid = str(policy.get("id", "")).strip()
        pass_test = str(policy.get("pass_test", "")).strip()
        fail_test = str(policy.get("fail_test", "")).strip()
        is_hard = bool(policy.get("hard", False)) or pid in hard

        pass_ok = bool(pass_test) and rg_exists(pass_test)
        fail_ok = bool(fail_test) and rg_exists(fail_test)
        status = "PASS" if pass_ok and fail_ok else "FAIL"
        if is_hard and status == "PASS":
            covered_hard += 1
        if not pass_ok:
            violations.append(f"{pid}: missing pass test reference `{pass_test}`")
        if not fail_ok:
            violations.append(f"{pid}: missing fail test reference `{fail_test}`")
        if pass_test == fail_test:
            violations.append(f"{pid}: pass/fail tests must be distinct")
        rows.append((pid, "hard" if is_hard else "soft", pass_test, fail_test, status))

    hard_percent = 100 if total_hard == 0 else int((covered_hard / total_hard) * 100)

    lines = [
        "# Policy Enforcement Status",
        "",
        "- Owner: `atlas-platform`",
        "- Generated from: `configs/policy/policy-enforcement-coverage.json`",
        f"- Hard policy coverage: `{covered_hard}/{total_hard}` (`{hard_percent}%`)",
        "",
        "| Policy | Class | Pass Test | Fail Test | Status |",
        "| --- | --- | --- | --- | --- |",
    ]
    for pid, klass, p, f, status in sorted(rows, key=lambda r: r[0]):
        lines.append(f"| `{pid}` | `{klass}` | `{p}` | `{f}` | `{status}` |")

    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text("\n".join(lines) + "\n")
    print(f"wrote {OUT}")

    if args.enforce:
        if hard_percent < 100:
            violations.append("hard policy coverage must be 100%")
        if violations:
            for v in violations:
                print(f"policy-enforcement violation: {v}", file=sys.stderr)
            return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
