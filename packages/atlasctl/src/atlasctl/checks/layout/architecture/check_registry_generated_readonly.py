#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[7]


def main() -> int:
    from atlasctl.checks.registry_legacy.ssot import generate_registry_json

    try:
        _out, changed = generate_registry_json(ROOT, check_only=True)
    except Exception as exc:  # noqa: BLE001
        print(f"registry generated check failed: {exc}")
        return 1
    if changed:
        print("registry drift detected; run ./bin/atlasctl dev regen-registry")
        return 1
    print("registry generated read-only check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
