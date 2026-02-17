#!/usr/bin/env bash
# Purpose: enforce artifacts git policy (ignored outputs, no tracked payloads).
# Inputs: .gitignore and git index.
# Outputs: non-zero on policy violation.
set -euo pipefail

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
cd "$ROOT"

if ! rg -n '^/artifacts/$' .gitignore >/dev/null 2>&1; then
  echo "artifacts policy failed: .gitignore must include /artifacts/" >&2
  exit 1
fi

tracked="$(git ls-files artifacts || true)"
if [ -n "$tracked" ]; then
  bad="$(printf '%s\n' "$tracked" | grep -Ev '^artifacts/\.gitkeep$' || true)"
  if [ -n "$bad" ]; then
    echo "artifacts policy failed: tracked artifacts payloads are forbidden" >&2
    printf '%s\n' "$bad" >&2
    exit 1
  fi
fi

echo "artifacts policy check passed"
