#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

"$ROOT_DIR/tutorials/scripts/validate_example_dataset.py" "$ROOT_DIR/configs/examples/datasets/atlas-example-minimal"
"$ROOT_DIR/tutorials/scripts/reproducibility_check.sh"
"$ROOT_DIR/tutorials/scripts/integrity_check.sh"

echo "tutorial CLI workflow completed"
