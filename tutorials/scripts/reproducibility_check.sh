#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DATASET_DIR="$ROOT_DIR/configs/examples/datasets/atlas-example-minimal"

first_hash=$(shasum -a 256 "$DATASET_DIR/genes.jsonl" | awk '{print $1}')
second_hash=$(shasum -a 256 "$DATASET_DIR/genes.jsonl" | awk '{print $1}')

if [[ "$first_hash" != "$second_hash" ]]; then
  echo "reproducibility check failed"
  exit 1
fi

echo "reproducibility check passed: $first_hash"
