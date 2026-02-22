#!/usr/bin/env python3
# Purpose: format ops YAML/JSON deterministically.
from __future__ import annotations

import json
import shutil
import subprocess
from pathlib import Path


def _repo_root() -> Path:
    return Path(__file__).resolve().parents[7]


def _format_json(root: Path) -> None:
    for base in (root / "ops", root / "configs/ops"):
        if not base.exists():
            continue
        for path in sorted(base.rglob("*.json")):
            if "/_generated/" in path.as_posix():
                continue
            try:
                data = json.loads(path.read_text(encoding="utf-8"))
            except Exception:
                continue
            path.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def _format_yaml(root: Path) -> None:
    yq = shutil.which("yq")
    if not yq:
        print("yq not installed; YAML formatting skipped")
        return
    for base in (root / "ops", root / "configs/ops"):
        if not base.exists():
            continue
        for path in sorted(list(base.rglob("*.yaml")) + list(base.rglob("*.yml"))):
            if "/_generated/" in path.as_posix():
                continue
            subprocess.run([yq, "-P", "-i", ".", str(path)], check=True, cwd=root)


def main() -> int:
    root = _repo_root()
    _format_json(root)
    _format_yaml(root)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
