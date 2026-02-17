#!/usr/bin/env sh
set -eu
. "$(dirname "$0")/common.sh"
need helm; need kubectl

TMP_VALUES="$(mktemp)"
cat > "$TMP_VALUES" <<YAML
rollout:
  enabled: true
YAML

helm template "$RELEASE" "$CHART" -n "$NS" -f "$VALUES" -f "$TMP_VALUES" | grep -q "kind: Rollout"

echo "rollout gate passed"
