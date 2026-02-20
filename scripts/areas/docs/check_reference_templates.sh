#!/usr/bin/env sh
# Purpose: script interface entrypoint.
# Inputs: command-line args and repository files/env as documented by caller.
# Outputs: exit status and deterministic stdout/stderr or generated artifacts.
set -eu

for f in $(find docs/reference -type f -name '*.md' ! -name 'INDEX.md' | sort); do
  for sec in '## What' '## Why' '## Scope' '## Non-goals' '## Contracts' '## Failure modes' '## How to verify' '## See also'; do
    grep -q "^$sec" "$f" || { echo "$f missing section: $sec" >&2; exit 1; }
  done
done

echo "reference templates check passed"