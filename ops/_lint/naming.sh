#!/usr/bin/env bash
# DIR_BUDGET_SHIM
set -euo pipefail
SCRIPT_DIR="$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)"
exec "$SCRIPT_DIR/docs/naming.sh" "$@"
