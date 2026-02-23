from __future__ import annotations

import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[8]
BASE = ROOT / "packages" / "atlasctl" / "src" / "atlasctl" / "commands" / "ops"
PATTERN = re.compile(r"ops/_generated_committed/")
ALLOW = {
    "packages/atlasctl/src/atlasctl/commands/ops/runtime_modules/ops_runtime_run.py",
    "packages/atlasctl/src/atlasctl/commands/ops/runtime_modules/ops_runtime_commands.py",
    "packages/atlasctl/src/atlasctl/commands/ops/runtime_modules/assets/root-lanes.sh",
    "packages/atlasctl/src/atlasctl/commands/ops/runtime_modules/assets/root-local.sh",
    "packages/atlasctl/src/atlasctl/commands/ops/lint/layout/json_schema_coverage.py",
    "packages/atlasctl/src/atlasctl/commands/ops/lint/layout/no_shadow_configs.py",
}


def _scan_files() -> list[Path]:
    files = list(BASE.rglob("*.py"))
    files.extend((BASE / "runtime_modules" / "assets").rglob("*.sh"))
    return sorted({p for p in files if p.is_file()})


def main() -> int:
    errs: list[str] = []
    for path in _scan_files():
        rel = path.relative_to(ROOT).as_posix()
        text = path.read_text(encoding="utf-8", errors="ignore")
        if PATTERN.search(text) and rel not in ALLOW:
            errs.append(f"{rel}: writes/refs ops/_generated_committed outside explicit update commands")
    if errs:
        print("\n".join(errs), file=sys.stderr)
        return 1
    print("ops/_generated_committed writes are limited to explicit update commands")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
