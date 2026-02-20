#!/usr/bin/env python3
from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
PROFILES = ROOT / "ops/obs/contract/goldens/profiles.json"


def main() -> int:
    payload = json.loads(PROFILES.read_text(encoding="utf-8"))
    errors: list[str] = []
    profiles = payload.get("profiles", {})
    for profile in ("local", "perf", "offline"):
        if profile not in profiles:
            errors.append(f"missing golden profile: {profile}")
            continue
        spec = profiles[profile]
        for key in ("metrics_golden", "trace_golden"):
            rel = spec.get(key)
            if not isinstance(rel, str) or not rel:
                errors.append(f"profile {profile} missing {key}")
                continue
            path = ROOT / rel
            if not path.exists():
                errors.append(f"profile {profile} missing file: {rel}")
            elif path.stat().st_size == 0:
                errors.append(f"profile {profile} has empty file: {rel}")
            elif key == "trace_golden":
                try:
                    json.loads(path.read_text(encoding="utf-8"))
                except Exception as exc:  # pragma: no cover - deterministic gate failure path
                    errors.append(f"profile {profile} invalid json in {rel}: {exc}")

    if errors:
        print("obs profile goldens check failed:", file=sys.stderr)
        for e in errors:
            print(f"- {e}", file=sys.stderr)
        return 1
    print("obs profile goldens check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
