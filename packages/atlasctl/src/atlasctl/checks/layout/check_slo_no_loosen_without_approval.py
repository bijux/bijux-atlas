#!/usr/bin/env python3
# Purpose: fail CI when SLOs are loosened without explicit approval file update.
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]
SLO_FILE = "configs/ops/slo/slo.v1.json"
APPROVAL_FILE = "configs/ops/slo/LOOSENING_APPROVAL.json"


def git_text(rev: str, path: str) -> str | None:
    proc = subprocess.run(["git", "show", f"{rev}:{path}"], cwd=ROOT, capture_output=True, text=True)
    return proc.stdout if proc.returncode == 0 else None


def changed_files(base: str) -> set[str]:
    proc = subprocess.run(["git", "diff", "--name-only", f"{base}..HEAD"], cwd=ROOT, capture_output=True, text=True)
    if proc.returncode != 0:
        return set()
    return {line.strip() for line in proc.stdout.splitlines() if line.strip()}


def is_loosen(old: dict, new: dict) -> list[str]:
    issues: list[str] = []
    old_slos = {row.get("id"): row for row in old.get("slos", []) if isinstance(row, dict)}
    new_slos = {row.get("id"): row for row in new.get("slos", []) if isinstance(row, dict)}

    for sid, old_row in old_slos.items():
        new_row = new_slos.get(sid)
        if not isinstance(new_row, dict):
            continue

        old_target = old_row.get("target")
        new_target = new_row.get("target")
        if isinstance(old_target, (int, float)) and isinstance(new_target, (int, float)) and new_target < old_target:
            issues.append(f"{sid}: target lowered {old_target} -> {new_target}")

        old_thr = old_row.get("threshold")
        new_thr = new_row.get("threshold")
        if isinstance(old_thr, dict) and isinstance(new_thr, dict):
            op_old = old_thr.get("operator")
            op_new = new_thr.get("operator")
            val_old = old_thr.get("value")
            val_new = new_thr.get("value")
            if op_old == op_new and isinstance(val_old, (int, float)) and isinstance(val_new, (int, float)):
                if op_new == "lt" and val_new > val_old:
                    issues.append(f"{sid}: threshold loosened (lt) {val_old} -> {val_new}")
                if op_new == "gt" and val_new < val_old:
                    issues.append(f"{sid}: threshold loosened (gt) {val_old} -> {val_new}")
    return issues


def main() -> int:
    head_parent = subprocess.run(["git", "rev-parse", "HEAD~1"], cwd=ROOT, capture_output=True, text=True)
    if head_parent.returncode != 0:
        print("slo no-loosen guard skipped: no parent commit available")
        return 0
    base = head_parent.stdout.strip()

    changed = changed_files(base)
    if SLO_FILE not in changed:
        print("slo no-loosen guard passed: no SLO changes")
        return 0

    old_text = git_text(base, SLO_FILE)
    new_text = (ROOT / SLO_FILE).read_text(encoding="utf-8")
    if old_text is None:
        print("slo no-loosen guard: baseline SLO file unavailable; require approval file")
        if APPROVAL_FILE not in changed:
            print(f"missing required approval update: {APPROVAL_FILE}", file=sys.stderr)
            return 1
        return 0

    old_cfg = json.loads(old_text)
    new_cfg = json.loads(new_text)
    loosening = is_loosen(old_cfg, new_cfg)
    if not loosening:
        print("slo no-loosen guard passed: no loosening detected")
        return 0

    if APPROVAL_FILE not in changed:
        print("slo no-loosen guard failed: loosened SLO without approval update", file=sys.stderr)
        for item in loosening:
            print(f"- {item}", file=sys.stderr)
        print(f"required changed file: {APPROVAL_FILE}", file=sys.stderr)
        return 1

    approval = json.loads((ROOT / APPROVAL_FILE).read_text(encoding="utf-8"))
    if approval.get("approved") is not True:
        print(f"{APPROVAL_FILE}: approved must be true when SLOs are loosened", file=sys.stderr)
        return 1

    print("slo no-loosen guard passed with explicit approval")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
