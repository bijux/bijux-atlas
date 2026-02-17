#!/usr/bin/env sh
set -eu
NS="${ATLAS_NS:-atlas-e2e}"
MODE="${1:-on}"
if [ "$MODE" = "on" ]; then
cat <<YAML | kubectl apply -f -
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: block-minio-egress
  namespace: ${NS}
spec:
  podSelector:
    matchLabels:
      app.kubernetes.io/name: bijux-atlas
  policyTypes: [Egress]
  egress:
    - to:
        - namespaceSelector:
            matchLabels:
              kubernetes.io/metadata.name: ${NS}
          podSelector:
            matchLabels:
              app: minio
      ports:
        - protocol: TCP
          port: 9000
YAML
else
  kubectl -n "$NS" delete networkpolicy block-minio-egress --ignore-not-found >/dev/null
fi
