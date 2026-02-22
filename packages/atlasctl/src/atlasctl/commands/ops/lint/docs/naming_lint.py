#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from pathlib import Path


def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for parent in cur.parents:
        if all((parent / marker).exists() for marker in ("makefiles", "packages", "configs", "ops")):
            return parent
    raise RuntimeError("unable to resolve repo root")


ROOT = _repo_root()


def _iter_ops_paths() -> list[Path]:
    out: list[Path] = []
    for path in (ROOT / "ops").rglob("*"):
        rel = path.relative_to(ROOT).as_posix()
        if rel.startswith("ops/_artifacts") or rel.startswith("ops/_generated"):
            continue
        if rel.startswith("ops/stack/minio"):
            continue
        out.append(path)
    return out


def main() -> int:
    bad = False
    banned = re.compile(r"(mid-load|phase|step|task|stage)")

    for path in _iter_ops_paths():
        name = path.name
        if banned.search(name):
            print(f"forbidden temporal/process token in path: {path.relative_to(ROOT).as_posix()}", file=sys.stderr)
            bad = True

    exts = {".sh", ".json", ".yaml", ".yml"}
    for path in sorted((ROOT / "ops").rglob("*")):
        if not path.is_file() or path.suffix not in exts:
            continue
        rel = path.relative_to(ROOT).as_posix()
        if rel.startswith("ops/stack/minio") or rel.startswith("ops/_artifacts") or rel.startswith("ops/_generated"):
            continue
        if "minio" in path.name:
            print(
                f"durable names must use 'store' (minio allowed only in ops/stack/minio): {rel}",
                file=sys.stderr,
            )
            bad = True

    for root in (ROOT / "scripts" / "areas" / "public" / "contracts",):
        if not root.exists():
            continue
        for path in sorted(root.rglob("*.sh")):
            if not re.fullmatch(r"[a-z0-9]+(?:-[a-z0-9]+)*\.sh", path.name):
                print(f"public shell script must use kebab-case: {path.relative_to(ROOT).as_posix()}", file=sys.stderr)
                bad = True
    run_dir = ROOT / "ops" / "run"
    if run_dir.exists():
        for path in sorted(run_dir.rglob("*.sh")):
            if not re.fullmatch(r"[a-z0-9]+(?:-[a-z0-9]+)*\.sh", path.name):
                print(f"public shell script must use kebab-case: {path.relative_to(ROOT).as_posix()}", file=sys.stderr)
                bad = True

    return 1 if bad else 0


if __name__ == "__main__":
    raise SystemExit(main())
