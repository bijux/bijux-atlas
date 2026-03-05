#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 ]]; then
  echo "usage: $0 <dataset-dir>"
  exit 2
fi

DATASET_DIR="$1"
if [[ ! -d "$DATASET_DIR" ]]; then
  echo "dataset directory not found: $DATASET_DIR"
  exit 2
fi

echo "ingest command preview for $DATASET_DIR"
echo "bijux-dev-atlas ops runtime execute --config configs/examples/runtime/server-minimal.toml"
echo "bijux-dev-atlas api verify --openapi configs/openapi/v1/openapi.generated.json"
