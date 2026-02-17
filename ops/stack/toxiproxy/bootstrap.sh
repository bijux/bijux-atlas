#!/usr/bin/env sh
set -eu
NS="${NS:-atlas-e2e}"
kubectl -n "$NS" run toxiproxy-bootstrap --restart=Never --rm -i \
  --image=curlimages/curl:8.10.1 --command -- sh -ceu '
api="http://toxiproxy.'"$NS"'.svc.cluster.local:8474"
# upsert minio proxy
curl -fsS -X DELETE "$api/proxies/minio" >/dev/null 2>&1 || true
curl -fsS -X POST "$api/proxies" \
  -H "Content-Type: application/json" \
  -d "{\"name\":\"minio\",\"listen\":\"0.0.0.0:19000\",\"upstream\":\"minio.'"$NS"'.svc.cluster.local:9000\"}" >/dev/null
'
