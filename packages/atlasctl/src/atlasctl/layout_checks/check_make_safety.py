#!/usr/bin/env python3
# Purpose: enforce make safety constraints (cargo target dir + root write policy).
# Inputs: Makefile and makefiles/*.mk.
# Outputs: non-zero if unsafe cargo invocations or root writes are detected.
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]
mk_files = [ROOT / "Makefile", *sorted((ROOT / "makefiles").glob("*.mk"))]

violations: list[str] = []
text = (ROOT / "makefiles" / "env.mk").read_text(encoding="utf-8") if (ROOT / "makefiles" / "env.mk").exists() else ""
if "CARGO_TARGET_DIR ?=" not in text:
    violations.append("makefiles/env.mk: missing CARGO_TARGET_DIR default")

cargo_re = re.compile(r"(^|\s)cargo(\s|$)")
unsafe_write_re = re.compile(r"(?:^|\\s)(?:touch|mkdir\\s+-p|cat\\s+>\\s*|cp\\s+[^\\n]*\\s)([^\\s\\\"';]+)")

for path in mk_files:
    for idx, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
        stripped = line.strip()
        if not stripped.startswith("@") and not line.startswith("\t"):
            continue
        if cargo_re.search(line) and "CARGO_TARGET_DIR" not in line and "atlasctl env isolate" not in line and "cargo --version" not in line:
            # global env default is acceptable; this check prevents explicit unset patterns.
            if "CARGO_TARGET_DIR:-" in line:
                violations.append(f"{path.relative_to(ROOT)}:{idx}: cargo invocation uses fallback pattern")
        m = unsafe_write_re.search(line)
        if m:
            target = m.group(1)
            if target.startswith("artifacts/") or target.startswith("$${ISO_ROOT}") or target.startswith("$(ISO_ROOT)"):
                continue
            if target.startswith("ops/") or target.startswith("docs/") or target.startswith("configs/") or target.startswith("makefiles/"):
                continue
            # likely repo-root write
            if "/" not in target and not target.startswith("$"):
                violations.append(f"{path.relative_to(ROOT)}:{idx}: potential repo-root write target `{target}`")

if violations:
    print("make safety check failed:", file=sys.stderr)
    for v in violations:
        print(f"- {v}", file=sys.stderr)
    raise SystemExit(1)

print("make safety check passed")
