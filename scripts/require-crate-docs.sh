#!/usr/bin/env sh
set -eu

for crate in crates/*; do
  [ -d "$crate" ] || continue
  [ -d "$crate/docs" ] || { echo "missing $crate/docs/" >&2; exit 1; }
  [ -f "$crate/docs/INDEX.md" ] || { echo "missing $crate/docs/INDEX.md" >&2; exit 1; }
  [ -f "$crate/docs/ARCHITECTURE.md" ] || { echo "missing $crate/docs/ARCHITECTURE.md" >&2; exit 1; }
  [ -f "$crate/docs/PUBLIC_API.md" ] || { echo "missing $crate/docs/PUBLIC_API.md" >&2; exit 1; }
  [ -f "$crate/docs/EFFECTS.md" ] || { echo "missing $crate/docs/EFFECTS.md" >&2; exit 1; }
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
  grep -q '^## Purpose' "$crate/README.md" || { echo "$crate/README.md must include ## Purpose" >&2; exit 1; }
  grep -q '^## Public API' "$crate/README.md" || { echo "$crate/README.md must include ## Public API" >&2; exit 1; }
  grep -q '^## Boundaries' "$crate/README.md" || { echo "$crate/README.md must include ## Boundaries" >&2; exit 1; }
  grep -q '^## Effects' "$crate/README.md" || { echo "$crate/README.md must include ## Effects" >&2; exit 1; }
  grep -q '^## Telemetry' "$crate/README.md" || { echo "$crate/README.md must include ## Telemetry" >&2; exit 1; }
  grep -q '^## Tests' "$crate/README.md" || { echo "$crate/README.md must include ## Tests" >&2; exit 1; }
  grep -q '^## Benches' "$crate/README.md" || { echo "$crate/README.md must include ## Benches" >&2; exit 1; }
  grep -q '^## Docs index' "$crate/README.md" || { echo "$crate/README.md must include ## Docs index" >&2; exit 1; }
done

echo "crate docs contract OK"
