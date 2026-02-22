#!/usr/bin/env python3
from __future__ import annotations

import os
import shutil
import subprocess
import sys
from pathlib import Path


def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for parent in cur.parents:
        if all((parent / marker).exists() for marker in ("makefiles", "packages", "configs", "ops")):
            return parent
    raise RuntimeError("unable to resolve repo root")


def main() -> int:
    root = _repo_root()
    out_arg = sys.argv[1] if len(sys.argv) > 1 else ""
    out = Path(out_arg) if out_arg else (root / "artifacts/perf/store")
    tmp = root / "artifacts/perf/_tmp"
    dataset_release = "110"
    dataset_species = "homo_sapiens"
    dataset_assembly = "GRCh38"

    shutil.rmtree(out, ignore_errors=True)
    shutil.rmtree(tmp, ignore_errors=True)
    out.mkdir(parents=True, exist_ok=True)
    tmp.mkdir(parents=True, exist_ok=True)

    cmd = [
        "cargo",
        "run",
        "--quiet",
        "-p",
        "bijux-atlas-cli",
        "--bin",
        "bijux-atlas",
        "--",
        "atlas",
        "ingest",
        "--gff3",
        str(root / "crates/bijux-atlas-ingest/tests/ops/fixtures/tiny/genes.gff3"),
        "--fasta",
        str(root / "crates/bijux-atlas-ingest/tests/ops/fixtures/tiny/genome.fa"),
        "--fai",
        str(root / "crates/bijux-atlas-ingest/tests/ops/fixtures/tiny/genome.fa.fai"),
        "--output-root",
        str(out),
        "--release",
        dataset_release,
        "--species",
        dataset_species,
        "--assembly",
        dataset_assembly,
    ]
    with open(os.devnull, "w", encoding="utf-8") as devnull:
        proc = subprocess.run(cmd, check=False, stdout=devnull)
    if proc.returncode != 0:
        return proc.returncode

    catalog = f"""{{
  "datasets": [
    {{
      "dataset": {{
        "release": "{dataset_release}",
        "species": "{dataset_species}",
        "assembly": "{dataset_assembly}"
      }},
      "manifest_path": "{dataset_release}/{dataset_species}/{dataset_assembly}/derived/manifest.json",
      "sqlite_path": "{dataset_release}/{dataset_species}/{dataset_assembly}/derived/gene_summary.sqlite"
    }}
  ]
}}
"""
    (out / "catalog.json").write_text(catalog, encoding="utf-8")
    print(f"prepared perf store at {out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
