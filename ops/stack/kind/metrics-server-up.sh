#!/usr/bin/env bash
set -euo pipefail
if [ "${OPS_DRY_RUN:-0}" = "1" ]; then
  echo "DRY-RUN kubectl apply metrics-server manifests"
  exit 0
fi
kubectl apply -f https://github.com/kubernetes-sigs/metrics-server/releases/latest/download/components.yaml
kubectl -n kube-system patch deploy metrics-server --type='json' -p='[{"op":"add","path":"/spec/template/spec/containers/0/args/-","value":"--kubelet-insecure-tls"}]' >/dev/null 2>&1 || true
kubectl -n kube-system rollout status deploy/metrics-server --timeout=180s
echo "metrics-server installed"
