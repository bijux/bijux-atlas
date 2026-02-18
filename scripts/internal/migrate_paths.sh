#!/usr/bin/env sh
# Purpose: migrate legacy root-level operation paths to canonical ops/* paths.
# Inputs: tracked repository files.
# Outputs: rewritten file contents when --apply is passed.
set -eu

MODE="${1:-}"
if [ -z "$MODE" ]; then
  MODE="--dry-run"
fi

if [ "$MODE" != "--dry-run" ] && [ "$MODE" != "--apply" ]; then
  echo "usage: $0 [--dry-run|--apply]" >&2
  exit 2
fi

PATTERN='(^|[^A-Za-z0-9_])(e2e/|load/|observability/|openapi/|charts/bijux-atlas|bin/isolate|bin/require-isolate|datasets/real-datasets\.json|fixtures/medium/)'

if [ "$MODE" = "--dry-run" ]; then
  rg -n "$PATTERN" $(git ls-files) || true
  exit 0
fi

while IFS= read -r file; do
  [ -f "$file" ] || continue
  perl -0777 -i -pe '
    s{(^|[^A-Za-z0-9_])e2e/}{$1ops/e2e/}g;
    s{(^|[^A-Za-z0-9_])load/}{$1ops/load/}g;
    s{(^|[^A-Za-z0-9_])observability/}{$1ops/obs/}g;
    s{(^|[^A-Za-z0-9_])openapi/}{$1configs/openapi/}g;
    s{(^|[^A-Za-z0-9_])charts/bijux-atlas}{$1ops/k8s/charts/bijux-atlas}g;
    s{(^|[^A-Za-z0-9_])bin/isolate}{$1scripts/bin/isolate}g;
    s{(^|[^A-Za-z0-9_])bin/require-isolate}{$1scripts/bin/require-isolate}g;
    s{(^|[^A-Za-z0-9_])datasets/real-datasets\.json}{$1ops/datasets/real-datasets.json}g;
    s{(^|[^A-Za-z0-9_])fixtures/medium/}{$1ops/fixtures/medium/}g;
  ' "$file"
done <<EOF
$(git ls-files)
EOF

echo "path migration applied"
