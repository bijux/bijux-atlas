#!/usr/bin/env sh
set -eu

for d in $(find docs -type d | sort); do
  case "$d" in
    docs|docs/_assets|docs/_*|docs/_*/*) continue ;;
  esac
  has_md=$(find "$d" -maxdepth 1 -type f -name '*.md' | wc -l | tr -d ' ')
  [ "$has_md" -eq 0 ] && continue
  [ -f "$d/INDEX.md" ] || { echo "missing INDEX.md in $d" >&2; exit 1; }

  for sec in '## What' '## Why' '## Scope' '## Non-goals' '## Contracts' '## Failure modes' '## How to verify' '## See also'; do
    grep -q "^$sec" "$d/INDEX.md" || { echo "$d/INDEX.md missing section: $sec" >&2; exit 1; }
  done
done

echo "index pages check passed"
