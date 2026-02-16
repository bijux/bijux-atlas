#!/usr/bin/env sh
set -eu

for crate in crates/*; do
  [ -d "$crate" ] || continue
  [ -f "$crate/README.md" ] || { echo "missing $crate/README.md" >&2; exit 1; }
  [ -f "$crate/ARCHITECTURE.md" ] || { echo "missing $crate/ARCHITECTURE.md" >&2; exit 1; }
  grep -q '^## Architecture' "$crate/ARCHITECTURE.md" || {
    echo "$crate/ARCHITECTURE.md must include '## Architecture' section" >&2
    exit 1
  }
done

echo "crate docs contract OK"
