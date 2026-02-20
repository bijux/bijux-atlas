#!/usr/bin/env python3
# Purpose: ensure CI/nightly workflows use canonical make entrypoints.
from __future__ import annotations

from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parents[2]
WF = ROOT / ".github/workflows"
CI_MK = ROOT / "makefiles" / "ci.mk"


def make_runs(path: Path) -> list[str]:
    text = path.read_text(encoding="utf-8")
    return re.findall(r"run:\s*make\s+([^\n]+)", text)


def main() -> int:
    errs: list[str] = []
    ci_file = WF / "ci.yml"
    ci_runs = make_runs(ci_file)
    if not any(cmd.strip().startswith("ci") for cmd in ci_runs):
        errs.append("ci.yml must run `make ci`")

    for p in sorted(WF.glob("*.yml")):
        text = p.read_text(encoding="utf-8")
        if re.search(r"\b(make\s+)?(legacy/[A-Za-z0-9_./-]+|ops-[A-Za-z0-9-]+-legacy)\b", text):
            errs.append(f"{p.name} must not invoke legacy entrypoints")
        if "schedule:" not in text:
            continue
        if p.name == "dependency-lock.yml":
            continue
        runs = make_runs(p)
        if not any(cmd.strip().startswith("nightly") for cmd in runs):
            errs.append(f"{p.name} must run `make nightly`")

    ci_mk = CI_MK.read_text(encoding="utf-8")
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
