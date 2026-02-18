#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
"$ROOT/ops/stack/kind/down.sh"
"$ROOT/ops/stack/kind/up.sh"
