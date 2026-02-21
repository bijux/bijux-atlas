#!/usr/bin/env python3
# Purpose: ensure CI/nightly workflows use canonical make entrypoints.
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[7]
WF = ROOT / ".github/workflows"
DEV_MK = ROOT / "makefiles" / "dev.mk"
CI_MK = ROOT / "makefiles" / "ci.mk"
PRIMARY = {
    "root",
    "root-local",
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
}
ALLOWED_WORKFLOW_OVERRIDES = {
    "dependency-lock.yml": {"ci-init-iso-dirs", "ci-dependency-lock-refresh"},
}


def make_runs(path: Path) -> list[str]:
    text = path.read_text(encoding="utf-8")
    return re.findall(r"run:\s*make\s+([^\n]+)", text)


def main() -> int:
    errs: list[str] = []
    ci_file = WF / "ci.yml"
    ci_text = ci_file.read_text(encoding="utf-8")
    ci_runs = make_runs(ci_file)
    has_make_ci = any(cmd.strip().startswith("ci") for cmd in ci_runs)
    has_atlasctl_ci = re.search(r"\./bin/atlasctl\s+dev\s+ci\s+run\b", ci_text) is not None
    if not (has_make_ci or has_atlasctl_ci):
        errs.append("ci.yml must run canonical CI front door (`./bin/atlasctl dev ci run` or `make ci`)")

    for p in sorted(WF.glob("*.yml")):
        text = p.read_text(encoding="utf-8")
        if re.search(r"\b(make\s+)?(legacy/[A-Za-z0-9_./-]+|ops-[A-Za-z0-9-]+-legacy)\b", text):
            errs.append(f"{p.name} must not invoke legacy entrypoints")
        scoped_primary_check = p.name == "ci.yml" or "schedule:" in text
        if scoped_primary_check:
            for cmd in make_runs(p):
                target = cmd.strip().split()[0]
                if target in ALLOWED_WORKFLOW_OVERRIDES.get(p.name, set()):
                    continue
                if target not in PRIMARY:
                    errs.append(f"{p.name} uses non-primary make target: {target}")
        if "schedule:" not in text:
            continue
        if p.name == "dependency-lock.yml":
            continue
        runs = make_runs(p)
        if not any(cmd.strip().startswith("nightly") for cmd in runs):
            errs.append(f"{p.name} must run `make nightly`")

    dev_mk = DEV_MK.read_text(encoding="utf-8")
    ci_mk = CI_MK.read_text(encoding="utf-8")
    if re.search(r"\b(legacy/[A-Za-z0-9_./-]+|ops-[A-Za-z0-9-]+-legacy)\b", dev_mk):
        errs.append("makefiles/dev.mk must not reference legacy entrypoints")
    if re.search(r"\b(legacy/[A-Za-z0-9_./-]+|ops-[A-Za-z0-9-]+-legacy)\b", ci_mk):
        errs.append("makefiles/ci.mk must not reference legacy entrypoints")

    if errs:
        print("ci entrypoint contract check failed", file=sys.stderr)
        for err in errs:
            print(f"- {err}", file=sys.stderr)
        return 1
    print("ci entrypoint contract check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
