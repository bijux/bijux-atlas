#!/usr/bin/env bash
# Purpose: fail if forbidden legacy root path references appear in tracked text files.
# Inputs: tracked files under Makefile/makefiles/scripts/.github/docs/ops.
# Outputs: non-zero on forbidden path references.
set -euo pipefail

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
cd "$ROOT"

PATTERN='\./(charts|e2e|load|observability|datasets|fixtures)/|docs/operations/ops/'
if rg -n "$PATTERN" Makefile makefiles .github ops scripts \
  -g '!packages/atlasctl/src/atlasctl/checks/layout/shell/check_no_forbidden_paths.sh'; then
  echo "forbidden legacy path references found" >&2
  exit 1
fi

echo "forbidden path reference gate passed"
