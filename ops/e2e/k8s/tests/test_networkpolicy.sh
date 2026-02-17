#!/usr/bin/env sh
set -eu
. "$(dirname "$0")/common.sh"
need kubectl

wait_ready
kubectl -n "$NS" delete pod egress-probe --ignore-not-found >/dev/null 2>&1 || true
kubectl -n "$NS" run egress-probe --image=nicolaka/netshoot --restart=Never --command -- sh -ceu '
  # should work (allowed DNS egress)
  nslookup kubernetes.default.svc.cluster.local >/dev/null
  # should fail (default deny external egress)
  if nc -z -w 2 1.1.1.1 80; then
    echo "forbidden egress unexpectedly allowed" >&2
    exit 1
  fi
  exit 0
'
kubectl -n "$NS" wait --for=condition=Ready --timeout=120s pod/egress-probe || true
kubectl -n "$NS" logs egress-probe || true
kubectl -n "$NS" delete pod egress-probe --ignore-not-found >/dev/null 2>&1 || true

echo "networkpolicy gate passed"
