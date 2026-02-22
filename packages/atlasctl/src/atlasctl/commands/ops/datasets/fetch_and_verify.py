#!/usr/bin/env python3
from __future__ import annotations

import hashlib
import json
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
    _run(
        [
            "python3",
            "packages/atlasctl/src/atlasctl/checks/layout/domains/policies/check_dataset_manifest_lock.py",
        ],
        root,
    )
    lock_path = root / "ops/datasets/manifest.lock"
    if not lock_path.exists():
        print("missing ops/datasets/manifest.lock", file=sys.stderr)
        return 1

    fixtures = [
        root / "ops/fixtures/medium/v1/data/genes.gff3",
        root / "ops/fixtures/medium/v1/data/genome.fa",
        root / "ops/fixtures/medium/v1/data/genome.fa.fai",
    ]
    if not all(p.is_file() for p in fixtures):
        _run(["python3", "packages/atlasctl/src/atlasctl/commands/ops/datasets/fixtures/fetch_medium.py"], root)

    lock = json.loads(lock_path.read_text(encoding="utf-8"))
    manifest = json.loads((root / "ops/datasets/manifest.json").read_text(encoding="utf-8"))
    name_to_ds = {d["name"]: d for d in manifest.get("datasets", []) if isinstance(d, dict) and "name" in d}
    errs: list[str] = []
    for e in lock.get("entries", []):
        if not isinstance(e, dict):
            continue
        ds = name_to_ds.get(e.get("name"))
        if not isinstance(ds, dict):
            continue
        for key, expected in (e.get("checksums") or {}).items():
            rel = (ds.get("paths") or {}).get(key)
            if not rel or expected is None:
                continue
            p = root / str(rel)
            got = hashlib.sha256(p.read_bytes()).hexdigest() if p.exists() else None
            if got != expected:
                errs.append(f"checksum mismatch {rel}: {got} != {expected}")
    if errs:
        print("dataset checksum verification failed", file=sys.stderr)
        print("\n".join(errs), file=sys.stderr)
        return 1
    print("dataset checksum verification passed")
    _run(["python3", "packages/atlasctl/src/atlasctl/commands/ops/datasets/fixtures/fetch_real_datasets.py"], root)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
