#!/usr/bin/env sh
# Purpose: script interface entrypoint.
# Inputs: command-line args and repository files/env as documented by caller.
# Outputs: exit status and deterministic stdout/stderr or generated artifacts.
set -eu

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
MANIFEST="$ROOT/datasets/real-datasets.json"
OUT_ROOT="${ATLAS_REALDATA_ROOT:-$ROOT/artifacts/real-datasets}"
TMP="$OUT_ROOT/_downloads"

mkdir -p "$OUT_ROOT" "$TMP"

python3 - "$MANIFEST" "$OUT_ROOT" "$TMP" <<'PY'
import hashlib
import json
import pathlib
import shutil
import subprocess
import sys
import tarfile

manifest_path = pathlib.Path(sys.argv[1])
out_root = pathlib.Path(sys.argv[2])
tmp = pathlib.Path(sys.argv[3])
manifest = json.loads(manifest_path.read_text())

by_id = {d["id"]: d for d in manifest["datasets"]}

def dataset_dir(dataset_id: str) -> pathlib.Path:
    release, species, assembly = dataset_id.split("/")
    return out_root / release / species / assembly

for ds in manifest["datasets"]:
    did = ds["id"]
    ddir = dataset_dir(did)
    ddir.mkdir(parents=True, exist_ok=True)

    if ds["kind"] == "download":
        archive = tmp / ds["archive"]
        if not archive.exists():
            if pathlib.Path(ds["url"]).exists():
                shutil.copyfile(ds["url"], archive)
            else:
                subprocess.check_call(["curl", "-fsSL", ds["url"], "-o", str(archive)])

        sha = hashlib.sha256(archive.read_bytes()).hexdigest()
        if sha != ds["sha256"]:
            raise SystemExit(f"checksum mismatch for {did}: {sha} != {ds['sha256']}")

        with tarfile.open(archive, "r:gz") as t:
            t.extractall(ddir)

    elif ds["kind"] == "derived":
        src = dataset_dir(ds["derived_from"])
        if not src.exists():
            raise SystemExit(f"missing source dataset for derived entry {did}: {src}")
        shutil.copytree(src, ddir, dirs_exist_ok=True)
        subprocess.check_call([str(pathlib.Path(ds["transform"])) , str(ddir)])
    else:
        raise SystemExit(f"unknown dataset kind: {ds['kind']}")

print(f"real datasets ready in {out_root}")
PY