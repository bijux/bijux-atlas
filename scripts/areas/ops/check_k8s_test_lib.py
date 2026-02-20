#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
LIB = ROOT / "ops/k8s/tests/checks/_lib"


def main() -> int:
    files = sorted(p for p in LIB.glob("*.sh") if p.is_file())
    if len(files) > 10:
        print(f"k8s test lib contract failed: {LIB} has {len(files)} files (max 10)")
        return 1
    for f in files:
        text = f.read_text(encoding="utf-8")
        if "k8s-test-common.sh" not in text:
            print(f"k8s test lib contract failed: {f.relative_to(ROOT)} must source canonical k8s-test-common.sh")
            return 1
    print("k8s test lib contract passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
