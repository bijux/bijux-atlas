#!/usr/bin/env sh
set -eu
. "$(dirname "$0")/common.sh"
need helm; need kubectl

TMP_VALUES="$(mktemp)"
cat > "$TMP_VALUES" <<YAML
cache:
  pinnedDatasets:
    - 110/homo_sapiens/GRCh38
datasetWarmupJob:
  enabled: true
YAML
install_chart -f "$TMP_VALUES"
kubectl -n "$NS" wait --for=condition=complete --timeout=5m job/"$SERVICE_NAME-dataset-warmup"

echo "warmup job gate passed"
