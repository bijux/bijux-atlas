#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
rm -rf "$ROOT_DIR/tutorials/evidence/dataset-packages"
rm -rf "$ROOT_DIR/tutorials/evidence/generated"
mkdir -p "$ROOT_DIR/tutorials/evidence/generated"

echo "tutorial workspace cleaned"
