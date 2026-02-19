#!/usr/bin/env python3
# Purpose: forbid script-to-script calls outside approved library prefixes.
# Inputs: scripts/**/*.sh and scripts/**/*.py sources.
# Outputs: non-zero exit on disallowed relative script call patterns.
from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
allowed = (
    "scripts/layout/",
    "scripts/docs/",
    "scripts/contracts/",
    "scripts/public/observability/",
    "scripts/tooling/",
    "scripts/tools/",
    "scripts/public/perf/",
    "scripts/public/contracts/",
    "scripts/release/",
    "scripts/fixtures/",
    "scripts/bootstrap/",
    "scripts/policy/",
    "scripts/ops/",
    "scripts/public/require-crate-docs.sh",
    "scripts/public/no-network-unit-tests.sh",
    "scripts/public/check-cli-commands.sh",
    "scripts/public/policy-schema-drift.py",
    "scripts/public/check-allow-env-schema.py",
    "scripts/check/",
    "scripts/gen/",
    "scripts/ci/",
    "scripts/bin/",
    "scripts/lib/",
    "scripts/python/",
    "scripts/internal/effects-lint.sh",
    "scripts/internal/naming-intent-lint.sh",
    "scripts/internal/migrate_paths.sh",
    "scripts/internal/openapi-generate.sh",
    "scripts/internal/exec.sh",
    "scripts/internal/env_dump.sh",
    "scripts/generate_scripts_readme.py",
    "scripts/README.md",
    "scripts/INDEX.md",
    "scripts/check_no_root_dumping.sh",
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
