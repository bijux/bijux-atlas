#!/usr/bin/env python3
# Purpose: ensure CI wrapper workflow calls resolve to atlasctl entrypoints.
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[7]
WORKFLOWS = sorted((ROOT / ".github" / "workflows").glob("*.yml"))
DEV_MK = ROOT / "makefiles" / "dev.mk"
RUN_RE = re.compile(r"^\s*-\s*run:\s*(.+)\s*$")
CI_TARGET_RE = re.compile(r"^([A-Za-z0-9_./-]+):(?:\s|$)", re.M)


def _ci_targets() -> set[str]:
    return {target for target in CI_TARGET_RE.findall(DEV_MK.read_text(encoding="utf-8")) if target.startswith("ci")}


def main() -> int:
    ci_targets = _ci_targets()
    ci_mk = DEV_MK.read_text(encoding="utf-8")
    errors: list[str] = []
    for target in sorted(ci_targets):
        if f"{target}:" not in ci_mk or f"\n\t@./bin/atlasctl dev ci " not in ci_mk and target != "ci":
            if target == "ci" and "\n\t@./bin/atlasctl dev ci run --json" in ci_mk:
                continue
            errors.append(f"makefiles/dev.mk target `{target}` must delegate to ./bin/atlasctl dev ci ...")

    for workflow in WORKFLOWS:
        for lineno, line in enumerate(workflow.read_text(encoding="utf-8").splitlines(), start=1):
            match = RUN_RE.match(line)
            if not match:
                continue
            cmd = match.group(1).strip().strip('"')
            if not cmd.startswith("make "):
                continue
            target = cmd.split()[1]
            if target.startswith("ci") and target not in ci_targets:
                errors.append(f"{workflow.relative_to(ROOT)}:{lineno}: workflow references unknown CI wrapper `{target}`")

    if errors:
        print("workflow calls-atlasctl check failed", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1

    print("workflow calls-atlasctl check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
