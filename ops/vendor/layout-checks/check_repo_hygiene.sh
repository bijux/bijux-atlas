#!/usr/bin/env bash
# Purpose: enforce root hygiene (no tracked IDE/OS/build pollution).
# Inputs: git index and repository filesystem.
# Outputs: non-zero when forbidden files/dirs are found.
set -euo pipefail

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
cd "$ROOT"

fail=0

if git ls-files | grep -Eq '(^|/)\.DS_Store$'; then
  echo "tracked .DS_Store files are forbidden" >&2
  git ls-files | grep -E '(^|/)\.DS_Store$' >&2 || true
  fail=1
fi

if git ls-files | grep -Eq '^\.idea/'; then
  echo "tracked .idea files are forbidden" >&2
  git ls-files | grep -E '^\.idea/' >&2 || true
  fail=1
fi

if git ls-files | grep -Eq '^target/'; then
  echo "tracked target/ files are forbidden" >&2
  git ls-files | grep -E '^target/' >&2 || true
  fail=1
fi

if [ -n "${CI:-}" ] && [ -d "$ROOT/target" ]; then
  echo "root target/ directory must not exist in CI workspaces" >&2
  fail=1
fi

if find . -path './.git' -prune -o -name '.DS_Store' -print | grep -q .; then
  echo "workspace contains .DS_Store; remove it" >&2
  find . -path './.git' -prune -o -name '.DS_Store' -print >&2
  fail=1
fi

if [ "$fail" -ne 0 ]; then
  exit 1
fi

echo "repo hygiene check passed"
