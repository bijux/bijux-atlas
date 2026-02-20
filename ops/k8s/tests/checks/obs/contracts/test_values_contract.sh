#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
cd "$ROOT"
./bin/bijux-atlas contracts check --checks chart-values

echo "values contract gate passed"
