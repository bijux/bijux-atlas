#!/usr/bin/env sh
set -eu

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
cd "$ROOT"

cargo test -p bijux-atlas-server --test schema_evolution_regression

echo "schema evolution regression passed"
