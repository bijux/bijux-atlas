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


def _must_match(text: str, pattern: str, label: str) -> list[str]:
    return [] if re.search(pattern, text, flags=re.MULTILINE) else [f"missing/invalid default: {label}"]


def main() -> int:
    root = _repo_root()
    values = (root / "ops/k8s/charts/bijux-atlas/values.yaml").read_text(encoding="utf-8")
    errors: list[str] = []
    errors += _must_match(values, r"^\s*enableDebugDatasets:\s*false\b", "enableDebugDatasets=false")
    errors += _must_match(values, r"^\s*cachedOnlyMode:\s*false\b", "cachedOnlyMode=false")
    errors += _must_match(values, r"^\s*readOnlyFsMode:\s*false\b", "readOnlyFsMode=false")
    errors += _must_match(values, r"^\s*requestTimeoutMs:\s*5000\b", "requestTimeoutMs=5000")
    errors += _must_match(values, r"^\s*maxBodyBytes:\s*16384\b", "maxBodyBytes=16384")
    errors += _must_match(values, r"^\s*allowPrivilegeEscalation:\s*false\b", "allowPrivilegeEscalation=false")
    errors += _must_match(values, r"^\s*runAsNonRoot:\s*true\b", "runAsNonRoot=true")
    if "networkPolicy:" in values and not re.search(r"(?ms)^\s*networkPolicy:.*?^\s*enabled:\s*true\b", values):
        errors.append("networkPolicy.enabled default must be true")
    if "ingress:" in values and not re.search(r"(?ms)^\s*ingress:.*?^\s*enabled:\s*false\b", values):
        errors.append("ingress.enabled default must be false")
    if errors:
        for err in errors:
            print(err, file=sys.stderr)
        return 1
    print("defaults-safe gate passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
