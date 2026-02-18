#!/usr/bin/env bash
set -euo pipefail
age_min="${OPS_STALE_NAMESPACE_MINUTES:-240}"
now="$(date +%s)"
while IFS= read -r ns; do
  [ -z "$ns" ] && continue
  ts="$(kubectl get ns "$ns" -o jsonpath='{.metadata.creationTimestamp}' 2>/dev/null || true)"
  [ -z "$ts" ] && continue
  created="$(date -j -f '%Y-%m-%dT%H:%M:%SZ' "$ts" +%s 2>/dev/null || date -d "$ts" +%s 2>/dev/null || echo 0)"
  [ "$created" -eq 0 ] && continue
  age=$(( (now - created) / 60 ))
  if [ "$age" -ge "$age_min" ]; then
    echo "deleting stale namespace: $ns (age ${age}m)"
    kubectl delete ns "$ns" --ignore-not-found >/dev/null 2>&1 || true
  fi
done <<LIST
$(kubectl get ns -o jsonpath='{range .items[*]}{.metadata.name}{"\n"}{end}' | grep '^atlas-ops-' || true)
LIST
