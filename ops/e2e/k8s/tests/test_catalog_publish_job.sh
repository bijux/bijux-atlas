#!/usr/bin/env sh
set -eu
. "$(dirname "$0")/common.sh"
need helm

TMP_VALUES="$(mktemp)"
cat > "$TMP_VALUES" <<YAML
catalogPublishJob:
  enabled: true
YAML
helm template "$RELEASE" "$CHART" -n "$NS" -f "$VALUES" -f "$TMP_VALUES" | grep -q "kind: Job"
helm template "$RELEASE" "$CHART" -n "$NS" -f "$VALUES" -f "$TMP_VALUES" | grep -q "catalog-publish"

echo "catalog publish job gate passed"
