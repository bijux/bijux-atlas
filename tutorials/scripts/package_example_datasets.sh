#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SRC_DIR="$ROOT_DIR/configs/examples/datasets"
OUT_DIR="$ROOT_DIR/tutorials/evidence/dataset-packages"

mkdir -p "$OUT_DIR"

for dataset in atlas-example-minimal atlas-example-medium atlas-example-large-synthetic; do
  tar -czf "$OUT_DIR/${dataset}.tar.gz" -C "$SRC_DIR" "$dataset"
  shasum -a 256 "$OUT_DIR/${dataset}.tar.gz" > "$OUT_DIR/${dataset}.tar.gz.sha256"
done

echo "packaged datasets in $OUT_DIR"
