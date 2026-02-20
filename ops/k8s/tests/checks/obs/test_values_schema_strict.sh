#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
. "$SCRIPT_DIR/../_lib/common.sh"
setup_test_traps
need helm grep

bad_values="$(mktemp)"
cat >"$bad_values" <<'YAML'
nonexistentKeyForStrictSchema: true
YAML

if helm lint "$CHART" -f "$bad_values" >/tmp/values-schema-strict.out 2>&1; then
  echo "values schema strict check failed: unknown key unexpectedly accepted" >&2
  exit 1
fi
grep -Eiq "additional property|unknown field|nonexistentKeyForStrictSchema" /tmp/values-schema-strict.out

echo "values schema strict contract passed"
