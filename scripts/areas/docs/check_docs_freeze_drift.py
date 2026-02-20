#!/usr/bin/env python3
# Purpose: ensure docs freeze generation is deterministic without requiring a clean git tree.
# Inputs: generated docs paths and atlasctl contract generators.
# Outputs: non-zero exit when regeneration mutates target files.
from __future__ import annotations

import hashlib
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
TARGETS = [
    ROOT / "docs" / "_generated" / "contracts",
    ROOT / "docs" / "_generated" / "contracts" / "chart-contract-index.md",
    ROOT / "docs" / "_generated" / "openapi",
    ROOT / "docs" / "contracts" / "errors.md",
    ROOT / "docs" / "contracts" / "metrics.md",
    ROOT / "docs" / "contracts" / "tracing.md",
    ROOT / "docs" / "contracts" / "endpoints.md",
    ROOT / "docs" / "contracts" / "config-keys.md",
    ROOT / "docs" / "contracts" / "chart-values.md",
]


def file_hash(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def snapshot(path: Path) -> dict[str, str]:
    if path.is_file():
        return {str(path): file_hash(path)}
    if path.is_dir():
        out: dict[str, str] = {}
        for child in sorted(p for p in path.rglob("*") if p.is_file()):
            out[str(child)] = file_hash(child)
        return out
    return {}


def main() -> int:
    before: dict[str, str] = {}
    for target in TARGETS:
        before.update(snapshot(target))

    subprocess.run(["./bin/bijux-atlas", "contracts", "generate", "--generators", "artifacts"], cwd=ROOT, check=True)
    subprocess.run(["python3", "scripts/areas/docs/generate_chart_contract_index.py"], cwd=ROOT, check=True)

    after: dict[str, str] = {}
    for target in TARGETS:
        after.update(snapshot(target))

    if before != after:
        print("docs freeze failed: generated docs drift detected", file=sys.stderr)
        changed = sorted(set(before.keys()) | set(after.keys()))
        for path in changed:
            if before.get(path) != after.get(path):
                print(f"- {Path(path).relative_to(ROOT)}", file=sys.stderr)
        return 1

    print("docs freeze check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
