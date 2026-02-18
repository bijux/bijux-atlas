#!/usr/bin/env bash
set -euo pipefail
NS="${ATLAS_NS:-atlas-e2e}"
RATE_KBPS="${1:-128}"
if [ "${OPS_DRY_RUN:-0}" = "1" ]; then
  echo "DRY-RUN toxiproxy bandwidth ns=$NS rate_kbps=$RATE_KBPS"
  exit 0
fi

kubectl -n "$NS" run toxiproxy-throttle --restart=Never --rm -i \
  --image=curlimages/curl:8.10.1 --command -- sh -ceu '
api="http://toxiproxy.'"$NS"'.svc.cluster.local:8474"
curl -fsS -X POST "$api/proxies/minio/toxics" -H "Content-Type: application/json" \
  -d "{\"name\":\"bandwidth\",\"type\":\"bandwidth\",\"stream\":\"downstream\",\"attributes\":{\"rate\":'"$RATE_KBPS"'}}" >/dev/null || true
'

echo "toxiproxy bandwidth throttle set: ${RATE_KBPS}kbps"
