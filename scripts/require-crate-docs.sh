#!/usr/bin/env sh
set -eu

for crate in crates/*; do
  [ -d "$crate" ] || continue
  [ -d "$crate/docs" ] || { echo "missing $crate/docs/" >&2; exit 1; }
  [ -f "$crate/docs/INDEX.md" ] || { echo "missing $crate/docs/INDEX.md" >&2; exit 1; }
  [ -f "$crate/docs/ARCHITECTURE.md" ] || { echo "missing $crate/docs/ARCHITECTURE.md" >&2; exit 1; }
  [ -f "$crate/docs/PUBLIC_API.md" ] || { echo "missing $crate/docs/PUBLIC_API.md" >&2; exit 1; }
  [ -f "$crate/README.md" ] || { echo "missing $crate/README.md" >&2; exit 1; }

  grep -q '^## Architecture' "$crate/docs/ARCHITECTURE.md" || {
    echo "$crate/docs/ARCHITECTURE.md must include '## Architecture' section" >&2
    exit 1
  }

  grep -q 'docs/INDEX.md' "$crate/README.md" || {
    echo "$crate/README.md must link docs/INDEX.md" >&2
    exit 1
  }
  grep -q 'docs/PUBLIC_API.md' "$crate/README.md" || {
    echo "$crate/README.md must link docs/PUBLIC_API.md" >&2
    exit 1
  }
  grep -q 'docs/effects-and-boundaries.md' "$crate/README.md" || {
    echo "$crate/README.md must link effects-and-boundaries doc" >&2
    exit 1
  }
done

echo "crate docs contract OK"
