#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
. "$SCRIPT_DIR/../_lib/common.sh"
setup_test_traps
need helm; need kubectl

TMP_VALUES="$(mktemp)"
cat > "$TMP_VALUES" <<YAML
rollout:
  enabled: true
YAML

helm template "$RELEASE" "$CHART" -n "$NS" -f "$VALUES" -f "$TMP_VALUES" | grep -q "kind: Rollout"
RENDERED="$(helm template "$RELEASE" "$CHART" -n "$NS" -f "$VALUES" -f "$TMP_VALUES")"
printf '%s' "$RENDERED" | grep -q "setWeight: 10"
printf '%s' "$RENDERED" | grep -q "setWeight: 50"
if printf '%s' "$RENDERED" | grep -q "analysis:"; then
  printf '%s' "$RENDERED" | grep -q "templates:"
fi

echo "rollout gate passed"
