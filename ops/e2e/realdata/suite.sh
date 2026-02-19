#!/usr/bin/env bash
set -euo pipefail
DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
SCENARIO=""
SUITE="full"

while [ $# -gt 0 ]; do
  case "$1" in
    --suite)
      SUITE="${2:-full}"
      shift 2
      ;;
    --scenario)
      SCENARIO="${2:-}"
      shift 2
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 2
      ;;
  esac
done

if [ -n "$SCENARIO" ]; then
  script="$(python3 - <<PY
import json
from pathlib import Path
manifest = json.loads((Path("$DIR") / "scenarios.json").read_text(encoding="utf-8"))
scenario = "$SCENARIO"
for item in manifest.get("scenarios", []):
    if item.get("id") == scenario:
        print(item.get("script", ""))
        break
else:
    raise SystemExit(f"unknown realdata scenario: {scenario}")
PY
)"
  exec "$DIR/$script"
fi

case "$SUITE" in
  full)
    while IFS= read -r script; do
      [ -n "$script" ] || continue
      "$DIR/$script"
    done <<EOF
$(python3 - <<PY
import json
from pathlib import Path
manifest = json.loads((Path("$DIR") / "scenarios.json").read_text(encoding="utf-8"))
for item in manifest.get("scenarios", []):
    print(item.get("script", ""))
PY
)
EOF
    ;;
  *)
    echo "unknown realdata suite: $SUITE" >&2
    exit 2
    ;;
esac
