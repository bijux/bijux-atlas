#!/usr/bin/env sh
# Purpose: ensure artifacts tree only contains approved output paths.
# Inputs: artifacts/ filesystem state and allowlist patterns.
# Outputs: non-zero when unexpected paths are found.
set -eu

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
ALLOWLIST="$ROOT/configs/ops/artifacts-allowlist.txt"

[ -f "$ALLOWLIST" ] || { echo "missing allowlist: $ALLOWLIST" >&2; exit 1; }

if [ ! -d "$ROOT/artifacts" ]; then
  echo "artifacts allowlist check passed (artifacts/ absent)"
  exit 0
fi

if [ "${ARTIFACTS_ALLOWLIST_STRICT:-0}" != "1" ]; then
  echo "artifacts allowlist check skipped (non-strict mode)"
  exit 0
fi

fail=0
FILES_TMP="$(mktemp)"
trap 'rm -f "$FILES_TMP"' EXIT

if [ -d "$ROOT/artifacts/target" ]; then
  echo "unexpected artifact directory: artifacts/target" >&2
  fail=1
fi

find "$ROOT/artifacts" -path "$ROOT/artifacts/target" -prune -o -type f -print \
  | sed "s#$ROOT/##" | sort > "$FILES_TMP"

while IFS= read -r p; do
  [ -n "$p" ] || continue
  ok=0
  while IFS= read -r pat; do
    [ -z "$pat" ] && continue
    case "$pat" in
      \#*) continue ;;
    esac
    case "$p" in
      "$pat") ok=1; break ;;
    esac
  done < "$ALLOWLIST"

  if [ "$ok" -ne 1 ]; then
    echo "unexpected artifact path: $p" >&2
    fail=1
  fi
done < "$FILES_TMP"

if [ "$fail" -ne 0 ]; then
  exit 1
fi

echo "artifacts allowlist check passed"
