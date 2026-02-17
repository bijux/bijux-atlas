#!/usr/bin/env sh
# Purpose: apply deterministic repository path migrations.
# Inputs: tracked files in git index/worktree.
# Outputs: updated path references and compatibility layout checks.
set -eu

./scripts/migrate_paths.sh --apply
./scripts/layout/check_root_shape.sh

echo "layout migration completed"
