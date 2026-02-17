#!/usr/bin/env sh
set -eu

bad=$(find crates -type f \( -name 'helpers.rs' -o -name '*_helpers.rs' \))
if [ -n "$bad" ]; then
  echo "naming-intent-lint: helpers naming is forbidden; use intent names" >&2
  echo "$bad" >&2
  exit 1
fi

echo "naming intent lint passed"
