#!/usr/bin/env python3
# Purpose: ban direct cargo/pytest and internal make target invocations in workflows.
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[7]
WORKFLOWS = sorted((ROOT / ".github" / "workflows").glob("*.yml"))

CARGO_RE = re.compile(r"\bcargo\s+(?:test|fmt|clippy)\b")
PYTEST_RE = re.compile(r"\bpytest\b")
MAKE_INTERNAL_RE = re.compile(r"\bmake\s+[^#\n]*\binternal/[A-Za-z0-9_./-]+")
RUN_LINE_RE = re.compile(r"^\s*-\s*run:\s*(.+)\s*$")
APPROVED_MAKE_TARGETS = {
    "ci",
    "nightly",
    "fmt",
    "lint",
    "test",
    "audit",
    "docs",
    "ops",
    "k8s",
    "load",
    "obs",
    "doctor",
    "report",
    "ci-fast",
    "ci-all",
    "ci-contracts",
    "ci-docs",
    "ci-ops",
    "ci-release",
    "ci-release-all",
    "ci-init",
    "ci-artifacts",
    "ci-help",
    "docs",
    "ops-down",
    "ops-k6-version-check",
    "ops-perf-e2e",
    "ops-stack-smoke",
    "ops-tools-check",
    "ops-up",
    "ops/contract-check",
    "policies/boundaries-check",
    "release-update-compat-matrix",
}


def main() -> int:
    errors: list[str] = []
    for workflow in WORKFLOWS:
        text = workflow.read_text(encoding="utf-8")
        for lineno, line in enumerate(text.splitlines(), start=1):
            if CARGO_RE.search(line):
                errors.append(
                    f"{workflow.relative_to(ROOT)}:{lineno}: forbidden direct cargo invocation in workflow line"
                )
            if PYTEST_RE.search(line):
                errors.append(
                    f"{workflow.relative_to(ROOT)}:{lineno}: forbidden direct pytest invocation in workflow line"
                )
            if MAKE_INTERNAL_RE.search(line):
                errors.append(
                    f"{workflow.relative_to(ROOT)}:{lineno}: forbidden internal make target invocation in workflow line"
                )
            match = RUN_LINE_RE.match(line)
            if not match:
                continue
            cmd = match.group(1).strip().strip('"')
            if cmd.startswith("make "):
                parts = cmd.split()
                if len(parts) >= 2 and parts[1] not in APPROVED_MAKE_TARGETS:
                    errors.append(
                        f"{workflow.relative_to(ROOT)}:{lineno}: workflow make target not approved: `{parts[1]}`"
                    )

    if errors:
        print("workflow command ban check failed", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1

    print("workflow command ban check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
