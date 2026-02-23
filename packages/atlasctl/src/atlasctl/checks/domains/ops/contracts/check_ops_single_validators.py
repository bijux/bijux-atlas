#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from pathlib import Path

def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for base in (cur, *cur.parents):
        if (base / "makefiles").exists() and (base / "packages").exists():
            return base
    raise RuntimeError("unable to resolve repository root")


ROOT = _repo_root()
ops_mk = (ROOT / "makefiles" / "ops.mk").read_text(encoding="utf-8", errors="ignore")

checks = {
    "ops-stack-validate": "./packages/atlasctl/src/atlasctl/commands/ops/stack/validate.py",
    "ops-observability-pack-verify": "./bin/atlasctl run ./packages/atlasctl/src/atlasctl/commands/ops/observability/verify_pack.py",
    "ops-load-manifest-validate": "./bin/atlasctl run ./packages/atlasctl/src/atlasctl/commands/ops/load/contracts/validate_suite_manifest.py",
}
errors: list[str] = []
for target, cmd in checks.items():
    if target not in ops_mk or cmd not in ops_mk:
        errors.append(f"missing canonical validator wiring: {target} -> {cmd}")

blocked = [r"validate_pack\.sh", r"validate_stack\.sh", r"validate_suite.*\.py"]
allow = {
    "packages/atlasctl/src/atlasctl/commands/ops/stack/validate.py",
    "packages/atlasctl/src/atlasctl/commands/ops/observability/verify_pack.py",
    "packages/atlasctl/src/atlasctl/commands/ops/load/contracts/validate_suite_manifest.py",
}
for path in (ROOT / "ops").rglob("*"):
    if not path.is_file():
        continue
    rel = path.relative_to(ROOT).as_posix()
    for pat in blocked:
        if re.search(pat, rel) and rel not in allow:
            errors.append(f"forbidden duplicate validator entrypoint: {rel}")

if errors:
    print("ops single validator contract failed:", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)

print("ops single validator contract passed")
