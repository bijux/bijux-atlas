#!/usr/bin/env sh
set -eu

for crate in crates/*; do
  [ -d "$crate" ] || continue

  for file in "$crate/docs/INDEX.md" "$crate/docs/ARCHITECTURE.md" "$crate/docs/PUBLIC_API.md" "$crate/docs/EFFECTS.md" "$crate/README.md"; do
    [ -f "$file" ] || { echo "missing $file" >&2; exit 1; }
  done

  # README required sections
  for section in "## Purpose" "## Public API" "## Boundaries" "## Effects" "## Telemetry" "## Tests" "## Benches" "## Docs index"; do
    grep -q "^$section" "$crate/README.md" || { echo "$crate/README.md missing section: $section" >&2; exit 1; }
  done

  grep -q 'docs/INDEX.md' "$crate/README.md" || { echo "$crate/README.md must link docs/INDEX.md" >&2; exit 1; }
  grep -q 'docs/PUBLIC_API.md' "$crate/README.md" || { echo "$crate/README.md must link docs/PUBLIC_API.md" >&2; exit 1; }

  # root duplicates forbidden when docs exists
  [ ! -f "$crate/ARCHITECTURE.md" ] || { echo "$crate/ARCHITECTURE.md must not exist (use docs/ARCHITECTURE.md)" >&2; exit 1; }
  [ ! -f "$crate/PUBLIC_API.md" ] || { echo "$crate/PUBLIC_API.md must not exist (use docs/PUBLIC_API.md)" >&2; exit 1; }

  # crate docs index required sections
  for section in "## API stability" "## Invariants" "## Failure modes" "## How to extend"; do
    grep -q "^$section" "$crate/docs/INDEX.md" || { echo "$crate/docs/INDEX.md missing section: $section" >&2; exit 1; }
  done

done

echo "crate docs contract OK"
