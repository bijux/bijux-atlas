#!/usr/bin/env sh
set -eu
NS="${ATLAS_NS:-atlas-e2e}"
LATENCY_MS="${1:-0}"
JITTER_MS="${2:-0}"
kubectl -n "$NS" run toxiproxy-latency --restart=Never --rm -i \
  --image=curlimages/curl:8.10.1 --command -- sh -ceu '
api="http://toxiproxy.'"$NS"'.svc.cluster.local:8474"
# clear old toxics
for t in $$(curl -fsS "$api/proxies/minio" | sed -n "s/.*\"name\":\"\([^\"]*\)\".*/\1/p"); do :; done
curl -fsS -X POST "$api/proxies/minio/toxics" -H "Content-Type: application/json" \
  -d "{\"name\":\"latency\",\"type\":\"latency\",\"stream\":\"downstream\",\"attributes\":{\"latency\":'"$LATENCY_MS"',\"jitter\":'"$JITTER_MS"'}}" >/dev/null || true
'
