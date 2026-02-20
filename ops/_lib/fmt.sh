#!/usr/bin/env bash
# DIR_BUDGET_SHIM
set -euo pipefail
exec "$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)/log/fmt.sh" "$@"
