#!/usr/bin/env bash
# Purpose: apply deterministic repository path migrations.
# Inputs: tracked files in git index/worktree.
# Outputs: updated path references and compatibility layout checks.
set -euo pipefail

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"

"$ROOT/scripts/internal/migrate_paths.sh" --apply

remove_legacy_root_entry() {
  local path="$1"
  local abs="$ROOT/$path"

  if [ -L "$abs" ] || [ -f "$abs" ]; then
    rm -f "$abs"
    return 0
  fi

  if [ -d "$abs" ]; then
    if [ -z "$(find "$abs" -mindepth 1 -maxdepth 1 -print -quit)" ]; then
      rmdir "$abs"
      return 0
    fi
    echo "cannot remove $path: directory is not empty" >&2
    return 1
  fi
}

for legacy in charts e2e load observability datasets fixtures; do
  remove_legacy_root_entry "$legacy"
done

"$ROOT/scripts/layout/check_root_shape.sh"
"$ROOT/scripts/layout/check_forbidden_root_names.sh"
"$ROOT/scripts/layout/check_repo_hygiene.sh"

echo "layout migration completed"
