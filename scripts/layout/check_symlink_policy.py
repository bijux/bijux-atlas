#!/usr/bin/env python3
# Purpose: enforce repository symlink allowlist and documentation approval policy.
# Inputs: filesystem symlinks and docs/development/symlinks.md.
# Outputs: non-zero when undocumented or disallowed symlinks exist.
from __future__ import annotations

from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parents[2]
DOC = ROOT / "docs" / "development" / "symlinks.md"
if not DOC.exists():
    print(f"missing symlink policy doc: {DOC}", file=sys.stderr)
    raise SystemExit(1)

text = DOC.read_text(encoding="utf-8")
entry_re = re.compile(r"- `([^`]+)` -> `([^`]+)`: .*\(Approval: `([^`]+)`\)")
entries = {name: {"target": target, "approval": approval} for name, target, approval in entry_re.findall(text)}

allowed_non_root = {
    "ops/e2e/stack": "ops/stack",
}

symlinks = [p.name for p in ROOT.iterdir() if p.is_symlink()]
if (ROOT / "ops/e2e/stack").is_symlink():
    symlinks.append("ops/e2e/stack")

violations: list[str] = []
for rel in sorted(symlinks):
    p = ROOT / rel
    raw_target = Path(str(p.readlink()))
    if raw_target.is_absolute():
        violations.append(f"symlink `{rel}` must use relative target, found absolute `{raw_target}`")
    if not p.exists():
        violations.append(f"symlink `{rel}` is broken (target does not exist)")
        target = "<broken>"
        resolved = None
    else:
        resolved = p.resolve()
        try:
            target = resolved.relative_to(ROOT).as_posix()
        except ValueError:
            violations.append(f"symlink `{rel}` points outside repo: `{resolved}`")
            target = str(resolved)
    if "/" not in rel:
        if rel not in entries:
            violations.append(f"root symlink `{rel}` missing docs/development/symlinks.md entry")
            continue
        declared = entries[rel]
        if not declared["approval"].startswith("APPROVAL-"):
            violations.append(f"root symlink `{rel}` missing required approval token prefix APPROVAL-")
        if target != declared["target"]:
            violations.append(f"root symlink `{rel}` target drift: expected `{declared['target']}`, got `{target}`")
    else:
        expected = allowed_non_root.get(rel)
        if expected is None:
            violations.append(f"non-root symlink forbidden by policy: `{rel}`")
        elif target != expected:
            violations.append(f"non-root symlink `{rel}` target drift: expected `{expected}`, got `{target}`")

dockerfile = ROOT / "Dockerfile"
if not dockerfile.is_symlink():
    violations.append("root `Dockerfile` must be a symlink to `docker/Dockerfile`")

if violations:
    print("symlink policy check failed:", file=sys.stderr)
    for v in violations:
        print(f"- {v}", file=sys.stderr)
    raise SystemExit(1)

print("symlink policy check passed")
