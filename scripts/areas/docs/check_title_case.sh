#!/usr/bin/env sh
# Purpose: script interface entrypoint.
# Inputs: command-line args and repository files/env as documented by caller.
# Outputs: exit status and deterministic stdout/stderr or generated artifacts.
set -eu

allow='API|SSOT|ADR|K8s|k6|v1|CI|CLI|JSON|YAML|HMAC|SSRF|SLO|SDK|DNA|ETag|URL|GC|EMBL'

errors=0
for f in $(find docs -type f -name '*.md' | sort); do
  title=$(sed -n '1s/^# //p' "$f")
  [ -n "$title" ] || continue
  echo "$title" | rg -q '[A-Z]{4,}' && {
    echo "$title" | rg -q -e "$allow" || { echo "non-Title-Case/shouting title in $f: $title" >&2; errors=1; }
  }
done
[ "$errors" -eq 0 ] || exit 1

echo "title case check passed"