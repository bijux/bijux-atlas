#!/usr/bin/env bash
# Purpose: fail when unexpected root-level files appear.
set -euo pipefail

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
ALLOWLIST="$ROOT/configs/repo/root-files-allowlist.txt"

if [ ! -f "$ALLOWLIST" ]; then
  echo "missing allowlist: $ALLOWLIST" >&2
  exit 1
fi

fail=0
actual="$(find "$ROOT" -maxdepth 1 -type f -print | sed "s#^$ROOT/##" | sort -u)"
allowed="$(grep -v '^#' "$ALLOWLIST" | sed '/^$/d' | sort -u)"
while IFS= read -r f; do
  [ -n "$f" ] || continue
  if ! printf '%s\n' "$allowed" | grep -qx "$f"; then
    echo "unexpected root file: $f" >&2
    fail=1
  fi
done <<EOF
$actual
EOF

if [ "$fail" -ne 0 ]; then
  echo "update configs/repo/root-files-allowlist.txt if intentional" >&2
  exit 1
fi

echo "root dumping check passed"
