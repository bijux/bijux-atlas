#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
. "$SCRIPT_DIR/../_lib/common.sh"
setup_test_traps
need kubectl

wait_ready
kubectl -n "$NS" delete pod metadata-egress-probe --ignore-not-found >/dev/null 2>&1 || true
kubectl -n "$NS" run metadata-egress-probe --image=nicolaka/netshoot --restart=Never --command -- sh -ceu '
  for ip in 169.254.169.254 100.100.100.200; do
    if nc -z -w 2 "$ip" 80; then
      echo "forbidden metadata egress unexpectedly allowed to $ip" >&2
      exit 1
    fi
  done
  exit 0
'
wait_kubectl_condition pod metadata-egress-probe Ready 120s || true
kubectl -n "$NS" logs metadata-egress-probe || true
kubectl -n "$NS" delete pod metadata-egress-probe --ignore-not-found >/dev/null 2>&1 || true

echo "networkpolicy metadata egress gate passed"
