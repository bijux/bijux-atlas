#!/usr/bin/env bash
# Purpose: apply deterministic repository path migrations.
# Inputs: tracked files in git index/worktree.
# Outputs: updated path references and compatibility layout checks.
set -euo pipefail

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"

"$ROOT/scripts/migrate_paths.sh" --apply

ensure_symlink() {
  local path="$1"
  local target="$2"
  local abs="$ROOT/$path"

  if [ -L "$abs" ]; then
    local current
    current="$(readlink "$abs")"
    if [ "$current" = "$target" ]; then
      return 0
    fi
    rm "$abs"
  elif [ -d "$abs" ]; then
    find "$abs" -mindepth 1 -maxdepth 1 -type l -exec rm {} +
    if [ -z "$(find "$abs" -mindepth 1 -maxdepth 1 -print -quit)" ]; then
      rmdir "$abs"
    else
      echo "cannot convert $path to symlink: non-compat files present" >&2
      return 1
    fi
  elif [ -f "$abs" ]; then
    rm "$abs"
  fi

  ln -s "$target" "$abs"
}

ensure_symlink "e2e" "ops/e2e"
ensure_symlink "load" "ops/load"
ensure_symlink "observability" "ops/observability"
ensure_symlink "bin" "scripts/bin"
ensure_symlink "charts" "ops/k8s/charts"
ensure_symlink "Dockerfile" "docker/Dockerfile"

"$ROOT/scripts/layout/check_root_shape.sh"
"$ROOT/scripts/layout/check_ops_canonical_shims.sh"
"$ROOT/scripts/layout/check_repo_hygiene.sh"

echo "layout migration completed"
