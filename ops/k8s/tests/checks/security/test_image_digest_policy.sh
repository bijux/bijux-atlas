#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
. "$SCRIPT_DIR/../_lib/common.sh"
setup_test_traps
need python3

python3 "$ROOT/scripts/areas/check/check-no-latest-tags.py"
"$SCRIPT_DIR/test_image_digest_pinning.sh"

echo "image digest policy contract passed"
