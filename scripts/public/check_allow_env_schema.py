#!/usr/bin/env python3
# owner: platform
# purpose: forbid ALLOW_* escape hatches unless declared in ops env schema.
# stability: public
# called-by: make policy-allow-env-lint, make ci-policy-allow-env-lint
from __future__ import annotations

import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SCHEMA = ROOT / "configs/ops/env.schema.json"
ALLOW_PATTERN = re.compile(r"\b(?:ATLAS_ALLOW_[A-Z0-9_]+|ALLOW_NON_KIND)\b")


def main() -> int:
    declared = set(json.loads(SCHEMA.read_text()).get("variables", {}).keys())
    rg = subprocess.run(
        ["rg", "-n", r"\b(?:ATLAS_ALLOW_[A-Z0-9_]+|ALLOW_NON_KIND)\b", "crates", "scripts", "makefiles", ".github", "docs"],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    violations: list[str] = []
    for line in rg.stdout.splitlines():
        parts = line.split(":", 2)
        if len(parts) < 3:
            continue
        path, line_no, text = parts
        for token in ALLOW_PATTERN.findall(text):
            if token not in declared:
                violations.append(f"{path}:{line_no}: undeclared ALLOW var `{token}`")
    if violations:
        for v in sorted(set(violations)):
            print(f"allow-env violation: {v}", file=sys.stderr)
        return 1
    print("allow-env schema lint passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
