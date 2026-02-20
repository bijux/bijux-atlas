#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
DIR="$ROOT/ops/obs/tests"
SUITES="$ROOT/ops/obs/suites/suites.json"
SUITE="${1:-}"
if [ "$SUITE" = "--suite" ]; then
  SUITE="${2:-full}"
fi
[ -n "$SUITE" ] || SUITE="full"

tests=()
while IFS= read -r line; do
  [ -n "$line" ] && tests+=("$line")
done <<EOF
$(python3 - <<PY
import json
s=json.load(open("$SUITES", encoding="utf-8"))
name="$SUITE"
for suite in s.get("suites", []):
    if suite.get("id") == name:
        for t in suite.get("tests", []):
            print(t)
        break
else:
    raise SystemExit(f"unknown suite id: {name}")
PY
)
EOF

for t in "${tests[@]}"; do
  "$DIR/$t"
done

echo "obs suite passed: $SUITE"
