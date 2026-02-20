#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path


def main() -> int:
    p = argparse.ArgumentParser()
    p.add_argument("--lock", required=True)
    p.add_argument("--out", required=True)
    args = p.parse_args()

    lock = Path(args.lock)
    lines = [ln.strip() for ln in lock.read_text(encoding="utf-8").splitlines() if ln.strip() and not ln.startswith("#")]
    packages = []
    for item in lines:
        name, version = item.split("==", 1)
        packages.append({"name": name, "version": version, "purl": f"pkg:pypi/{name}@{version}"})

    payload = {
        "schema_version": 1,
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "source_lock": lock.as_posix(),
        "package_count": len(packages),
        "packages": packages,
    }
    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(out.as_posix())
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
