#!/usr/bin/env python3
# Purpose: ensure generated contract markdown matches JSON registries.
# Inputs: docs/contracts/*.json and generated markdown outputs
# Outputs: non-zero exit on drift
from __future__ import annotations

import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]


def main() -> int:
    subprocess.run(["./scripts/areas/contracts/generate_contract_artifacts.py"], cwd=ROOT, check=True)
    subprocess.run(["python3", "scripts/areas/docs/generate_chart_contract_index.py"], cwd=ROOT, check=True)
    targets = [
        "docs/_generated/contracts",
        "docs/contracts/errors.md",
        "docs/contracts/metrics.md",
        "docs/contracts/tracing.md",
        "docs/contracts/endpoints.md",
        "docs/contracts/config-keys.md",
        "docs/contracts/chart-values.md",
    ]
    proc = subprocess.run(["git", "diff", "--", *targets], cwd=ROOT, capture_output=True, text=True, check=False)
    if proc.stdout.strip() or proc.stderr.strip():
        print("generated contract docs drift detected")
        print(proc.stdout)
        return 1
    print("generated contract docs check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
