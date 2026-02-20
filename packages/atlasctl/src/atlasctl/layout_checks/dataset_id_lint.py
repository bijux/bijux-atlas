#!/usr/bin/env python3
# owner: bijux-atlas-operations
# purpose: validate DatasetId and DatasetKey strings across ops fixtures/manifests.
# stability: public
# called-by: make dataset-id-lint
# Purpose: enforce DatasetId and DatasetKey formatting contract over ops fixture metadata.
# Inputs: repository fixture manifests under ops/ and scripts/layout policy constants.
# Outputs: exit 0 when all IDs are valid, else deterministic validation errors on stderr.
from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]
RELEASE_RE = re.compile(r"^[0-9]{1,16}$")
SPECIES_RE = re.compile(r"^[a-z0-9_]{1,64}$")
ASSEMBLY_RE = re.compile(r"^[A-Za-z0-9._]{1,64}$")


def validate_triplet(dataset_id: str) -> list[str]:
    errs: list[str] = []
    parts = dataset_id.split("/")
    if len(parts) != 3:
        return [f"invalid DatasetId `{dataset_id}`: expected release/species/assembly"]
    rel, species, assembly = parts
    if not RELEASE_RE.fullmatch(rel):
        errs.append(f"invalid release `{rel}` in `{dataset_id}`")
    if not SPECIES_RE.fullmatch(species) or species.startswith("_") or species.endswith("_") or "__" in species:
        errs.append(f"invalid species `{species}` in `{dataset_id}`")
    if not ASSEMBLY_RE.fullmatch(assembly):
        errs.append(f"invalid assembly `{assembly}` in `{dataset_id}`")
    return errs


def validate_key(dataset_key: str) -> list[str]:
    errs: list[str] = []
    m = re.fullmatch(r"release=([^&]+)&species=([^&]+)&assembly=([^&]+)", dataset_key)
    if not m:
        return [f"invalid DatasetKey `{dataset_key}`: expected release=<r>&species=<s>&assembly=<a>"]
    errs.extend(validate_triplet(f"{m.group(1)}/{m.group(2)}/{m.group(3)}"))
    return errs


def main() -> int:
    errors: list[str] = []
    manifest = json.loads((ROOT / "ops/datasets/manifest.json").read_text())
    for ds in manifest.get("datasets", []):
        did = ds.get("id", "")
        errors.extend(validate_triplet(did))

    real_manifest = ROOT / "ops/datasets/real-datasets.json"
    if real_manifest.exists():
        real = json.loads(real_manifest.read_text())
        for ds in real.get("datasets", []):
            did = ds.get("id", "")
            errors.extend(validate_triplet(did))

    pinned_queries = ROOT / "ops/load/queries/pinned-v1.json"
    if pinned_queries.exists():
        q = json.loads(pinned_queries.read_text())
        dataset_key = q.get("dataset", "")
        errors.extend(validate_key(dataset_key))

    if errors:
        print("dataset-id lint failed:", file=sys.stderr)
        for e in errors:
            print(f"- {e}", file=sys.stderr)
        return 1
    print("dataset-id lint passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
