#!/usr/bin/env bash
# owner: docs-governance
# purpose: fail when legacy temporal/task language appears in docs and observability assets.
# stability: public
# called-by: make docs, make docs-lint-names, make ci-docs-build
set -euo pipefail

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
ALLOWLIST="$ROOT/scripts/docs/legacy-terms-allowlist.txt"

if [ ! -f "$ALLOWLIST" ]; then
  echo "missing allowlist: $ALLOWLIST" >&2
  exit 2
fi

# Forbidden planning/placeholder patterns.
PATTERNS=(
  "\\bphase\\s+[0-9ivx]+\\b"
  "\\bphase\\s+stability\\b"
  "\\bphase\\s+contract\\b"
  "\\b(step|task|stage|iteration|round)\\s+[0-9ivx]+\\b"
  "\\bvnext\\s+placeholder\\b"
  "\\btemporary\\b" # ATLAS-EXC-0101: scanner pattern token for detecting forbidden legacy wording.
  "\\bwip\\b"
)

TMP="$(mktemp)"
trap 'rm -f "$TMP"' EXIT

for pattern in "${PATTERNS[@]}"; do
  rg -n -i -e "$pattern" \
    "$ROOT/docs" \
    "$ROOT/ops" \
    "$ROOT/crates" \
    --glob 'docs/**/*.md' \
    --glob 'ops/**/*.md' \
    --glob 'crates/**/docs/**/*.md' \
    --glob '!docs/_drafts/**' \
    --glob '!docs/_generated/**' \
    --glob '!**/CHANGELOG*' \
    --glob '!**/changelog*' >>"$TMP" || true
done

if [ ! -s "$TMP" ]; then
  echo "legacy language gate passed"
  exit 0
fi

FAIL=0
while IFS= read -r line; do
  skip=0
  text="${line#*:*:}"
  while IFS= read -r allow || [ -n "$allow" ]; do
    [ -z "$allow" ] && continue
    case "$allow" in
      \#*) continue ;;
    esac
    if printf '%s' "$text" | rg -qi --fixed-strings "$allow"; then
      skip=1
      break
    fi
  done < "$ALLOWLIST"

  if [ "$skip" -eq 0 ]; then
    printf '%s\n' "$line" >&2
    FAIL=1
  fi
done < "$TMP"

if [ "$FAIL" -ne 0 ]; then
  echo "legacy language gate failed; rewrite with stable terms (see docs/_style/stability-levels.md)" >&2
  exit 1
fi

echo "legacy language gate passed"
