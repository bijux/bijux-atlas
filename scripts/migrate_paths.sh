#!/usr/bin/env sh
# Purpose: migrate stale top-level operation paths to ops/* canonical paths.
# Inputs: tracked repository files.
# Outputs: updated file contents when --apply is passed.
set -eu

MODE="dry-run"
if [ "${1:-}" = "--apply" ]; then
  MODE="apply"
fi

PATTERNS='(?<!ops/)e2e/=>ops/e2e/ (?<!ops/)load/=>ops/load/ (?<!ops/)observability/=>ops/observability/ (?<!ops/)openapi/=>ops/openapi/'

FILES=$(git ls-files)

if [ "$MODE" = "dry-run" ]; then
  echo "stale path references:"
  printf '%s\n' "$FILES" | xargs rg -n "(^|[^a-zA-Z0-9_])(e2e/|load/|observability/|openapi/)" || true
  exit 0
fi

printf '%s\n' "$FILES" | while IFS= read -r f; do
  [ -f "$f" ] || continue
  case "$f" in
    artifacts/*|target/*) continue ;;
  esac
  perl -0777 -i -pe 's{(?<!ops/)e2e/}{ops/e2e/}g; s{(?<!ops/)load/}{ops/load/}g; s{(?<!ops/)observability/}{ops/observability/}g; s{(?<!ops/)openapi/}{ops/openapi/}g;' "$f"
done

echo "path migration applied"
