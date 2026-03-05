#!/usr/bin/env python3
from __future__ import annotations

import json
import pathlib
import sys

REQUIRED = {"gene_id", "symbol", "chromosome", "biotype", "length_bp"}


def validate(dataset_dir: pathlib.Path) -> int:
    metadata = dataset_dir / "metadata.json"
    genes = dataset_dir / "genes.jsonl"
    if not metadata.exists() or not genes.exists():
        print(f"missing metadata.json or genes.jsonl in {dataset_dir}")
        return 1

    meta = json.loads(metadata.read_text(encoding="utf-8"))
    required_meta = {"dataset_id", "schema_version", "record_count", "description"}
    if not required_meta.issubset(meta):
        print("metadata fields missing")
        return 1

    count = 0
    for line in genes.read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        row = json.loads(line)
        if not REQUIRED.issubset(row):
            print(f"row missing fields: {row}")
            return 1
        count += 1

    if count != int(meta["record_count"]):
        print(f"record_count mismatch metadata={meta['record_count']} actual={count}")
        return 1

    print(f"validation passed for {dataset_dir.name} ({count} rows)")
    return 0


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("usage: validate_example_dataset.py <dataset-dir>")
        raise SystemExit(2)
    raise SystemExit(validate(pathlib.Path(sys.argv[1])))
