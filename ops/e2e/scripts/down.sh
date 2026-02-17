#!/usr/bin/env sh
set -eu

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
"$ROOT/ops/stack/scripts/uninstall.sh"
echo "e2e stack is down"
