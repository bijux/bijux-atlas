#!/usr/bin/env bash
# Purpose: replace legacy root path references with canonical ops/* paths.
# Inputs: text files under selected repository trees.
# Outputs: in-place path updates (idempotent).
set -euo pipefail

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
MODE="${1:---apply}"
if [[ "$MODE" != "--apply" && "$MODE" != "--dry-run" ]]; then
  echo "usage: $0 [--apply|--dry-run]" >&2
  exit 2
fi

# Restricted scope per policy.
SCOPE=(Makefile makefiles scripts .github docs ops)

legacy_pattern='\./(charts|e2e|load|observability|datasets|fixtures)/'
if [[ "$MODE" == "--dry-run" ]]; then
  rg -n "$legacy_pattern|operations/ops/|docs/operations/ops/" "${SCOPE[@]}" || true
  exit 0
fi

while IFS= read -r file; do
  [[ -f "$file" ]] || continue
  perl -0777 -i -pe '
    s{\./charts/}{./ops/k8s/charts/}g;
    s{\./e2e/}{./ops/e2e/}g;
    s{\./load/}{./ops/load/}g;
    s{\./observability/}{./ops/observability/}g;
    s{\./datasets/}{./ops/datasets/}g;
    s{\./fixtures/}{./ops/fixtures/}g;
    s{docs/operations/ops/}{docs/operations/}g;
    s{operations/ops/}{operations/}g;
  ' "$file"
done < <(git ls-files "${SCOPE[@]}")

echo "canonical path replacement applied"
