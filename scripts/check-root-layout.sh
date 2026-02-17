#!/usr/bin/env sh
# Purpose: forbid unapproved top-level directories in repository root.
# Inputs: repository root directory entries.
# Outputs: non-zero exit when unexpected top-level dirs are present.
set -eu

ALLOWED_DIRS="
.cargo
.github
.idea
artifacts
bin
charts
configs
crates
datasets
docs
e2e
fixtures
load
makefiles
observability
ops
scripts
target
xtask
"

errors=0
for d in */; do
  name="${d%/}"
  [ -d "$name" ] || continue
  case "$ALLOWED_DIRS" in
    *"\n$name\n"*) : ;;
    *)
      echo "root layout check failed: unexpected top-level dir '$name'" >&2
      errors=1
      ;;
  esac
done

[ "$errors" -eq 0 ] || exit 1

echo "root layout check passed"
