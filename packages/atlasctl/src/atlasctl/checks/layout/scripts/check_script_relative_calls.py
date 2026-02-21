#!/usr/bin/env python3
# Purpose: forbid script-to-script calls outside approved library prefixes.
# Inputs: scripts/**/*.sh and scripts/**/*.py sources.
# Outputs: non-zero exit on disallowed relative script call patterns.
from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[6]
allowed = (
    "packages/atlasctl/src/atlasctl/checks/layout/",
        "packages/atlasctl/src/atlasctl/observability/contracts/",
    "scripts/tooling/",
    "scripts/areas/tools/",
    "scripts/areas/public/perf/",
    "scripts/areas/public/contracts/",
    "scripts/areas/release/",
    "ops/datasets/scripts/fixtures/",
    "scripts/areas/policy/",
    "scripts/areas/ops/",
    "scripts/areas/public/no-network-unit-tests.sh",
    "scripts/README.md",
    "scripts/INDEX.md",
    "packages/atlasctl/src/atlasctl/checks/layout/shell/check_no_root_dumping.sh",
    "ops/load/scripts/",
    "ops/obs/scripts/",
    "ops/run/",
    "ops/_lib/",
    "ops/e2e/scripts/",
    "ops/datasets/scripts/",
    "ops/stack/scripts/",
    "ops/k8s/scripts/",
    "ops/CONTRACT.md",
    "ops/e2e",
)

violations: list[str] = []
for path in sorted((ROOT / "scripts").rglob("*")):
    if not path.is_file() or path.suffix not in {".sh", ".py"}:
        continue
    text = path.read_text(encoding="utf-8", errors="ignore")
    for idx, line in enumerate(text.splitlines(), start=1):
        if "re.compile(" in line:
            continue
        if "$ROOT/" not in line and "./scripts/" not in line and "./ops/" not in line:
            continue
        for m in re.finditer(r"(?:\$ROOT/|\./)(scripts/[^\s\"']+|ops/[^\s\"']+)", line):
            target = m.group(1).rstrip(";")
            if "}" in target:
                # Ignore template/substitution fragments from sed/perl replacement strings.
                continue
            if any(target.startswith(prefix) for prefix in allowed):
                continue
            violations.append(f"{path.relative_to(ROOT)}:{idx}: {target}")

if violations:
    print("disallowed script relative calls found:")
    for v in violations:
        print(f"- {v}")
    raise SystemExit(1)
print("script relative-call gate passed")
