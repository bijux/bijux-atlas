#!/usr/bin/env python3
from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path


def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for parent in cur.parents:
        if all((parent / marker).exists() for marker in ("makefiles", "packages", "configs", "ops")):
            return parent
    raise RuntimeError("unable to resolve repo root")


def _run(cmd: list[str], cwd: Path) -> None:
    proc = subprocess.run(cmd, cwd=cwd, text=True, capture_output=True, check=False)
    if proc.stdout:
        print(proc.stdout, end="")
    if proc.returncode != 0:
        if proc.stderr:
            print(proc.stderr, end="", file=sys.stderr)
        raise SystemExit(proc.returncode)


def main() -> int:
    root = _repo_root()
    qc_cfg = os.environ.get("ATLAS_QC_THRESHOLDS", str(root / "configs/ops/dataset-qc-thresholds.v1.json"))
    report_dir = Path(os.environ.get("ATLAS_E2E_OUTPUT_ROOT", str(root / "artifacts/e2e-datasets")))
    release = os.environ.get("ATLAS_DATASET_RELEASE", "110")
    species = os.environ.get("ATLAS_DATASET_SPECIES", "homo_sapiens")
    assembly = os.environ.get("ATLAS_DATASET_ASSEMBLY", "GRCh38")
    qc_report = report_dir / f"release={release}" / f"species={species}" / f"assembly={assembly}" / "derived/qc.json"
    if not qc_report.is_file():
        print(f"dataset QC failed: missing qc report: {qc_report}", file=sys.stderr)
        return 1
    _run([
        "cargo", "run", "-q", "-p", "bijux-atlas-cli", "--bin", "bijux-atlas", "--",
        "atlas", "ingest-validate", "--qc-report", str(qc_report), "--thresholds", str(qc_cfg)
    ], root)
    _run([
        "python3", "packages/atlasctl/src/atlasctl/commands/ops/datasets/qc_summary.py",
        "--qc", str(qc_report), "--out", str(report_dir / "qc-summary.md")
    ], root)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
