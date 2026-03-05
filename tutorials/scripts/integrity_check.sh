#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DATASET_DIR="$ROOT_DIR/configs/examples/datasets/atlas-example-minimal"
MANIFEST="$ROOT_DIR/tutorials/contracts/atlas-example-minimal.sha256"

mkdir -p "$(dirname "$MANIFEST")"
shasum -a 256 "$DATASET_DIR/genes.jsonl" > "$MANIFEST"
shasum -a 256 -c "$MANIFEST"
