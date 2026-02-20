#!/usr/bin/env python3
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
CONFIGS = ROOT / "configs"
SKIP_PARTS = {"_schemas", "__pycache__", ".vale"}


def _is_skipped(path: Path) -> bool:
    rel = path.relative_to(ROOT)
    return any(part in SKIP_PARTS for part in rel.parts)


def _validate_yaml(path: Path) -> str | None:
    if path.suffix not in {".yaml", ".yml"}:
        return None
    cmd = [
        "python3",
        "-c",
        "import sys, yaml; yaml.safe_load(open(sys.argv[1], 'r', encoding='utf-8').read())",
        str(path),
    ]
    proc = subprocess.run(cmd, capture_output=True, text=True)
    if proc.returncode == 0:
        return None
    return proc.stderr.strip() or proc.stdout.strip() or "yaml parse failed"


def main() -> int:
    errors: list[str] = []
    for path in sorted(CONFIGS.rglob("*")):
        if not path.is_file() or _is_skipped(path):
            continue
        if path.suffix == ".json":
            try:
                json.loads(path.read_text(encoding="utf-8"))
            except Exception as exc:  # noqa: BLE001
                errors.append(f"{path.relative_to(ROOT)}: invalid json ({exc})")
            continue
        if path.suffix in {".yaml", ".yml"}:
            yaml_error = _validate_yaml(path)
            if yaml_error:
                errors.append(f"{path.relative_to(ROOT)}: invalid yaml ({yaml_error})")

    if errors:
        print("config file parse check failed", file=sys.stderr)
        for err in errors:
            print(f"- {err}", file=sys.stderr)
        return 1
    print("config file parse check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
