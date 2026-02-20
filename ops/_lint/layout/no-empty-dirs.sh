#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
cd "$ROOT"

bad=0
while IFS= read -r dir; do
  rel="${dir#./}"
  [[ "$rel" == "ops/_artifacts"* ]] && continue
  [[ "$rel" == "ops/_generated"* ]] && continue

  # Allow directories that only exist to carry an INDEX.md rationale.
  files=$(find "$dir" -mindepth 1 -maxdepth 1 ! -name '.DS_Store' | wc -l | tr -d ' ')
  if [[ "$files" -eq 0 ]]; then
    echo "empty directory: $rel" >&2
    bad=1
    continue
  fi

  non_index=$(find "$dir" -mindepth 1 -maxdepth 1 ! -name 'INDEX.md' ! -name '.DS_Store' | wc -l | tr -d ' ')
  if [[ "$non_index" -eq 0 ]] && [[ ! -f "$dir/INDEX.md" ]]; then
    echo "directory must include INDEX.md explanation: $rel" >&2
    bad=1
  fi
done < <(find ./ops -type d)

exit "$bad"
