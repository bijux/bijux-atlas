#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

if rg -n --glob '*.sh' --glob '*.mk' --glob '!ops/k8s/tests/checks/obs/test_helm_repo_pinning.sh' 'helm repo add' ops makefiles scripts >/dev/null; then
  echo "unpinned helm repo usage detected; use local chart path only" >&2
  rg -n --glob '*.sh' --glob '*.mk' --glob '!ops/k8s/tests/checks/obs/test_helm_repo_pinning.sh' 'helm repo add' ops makefiles scripts || true
  exit 1
fi

echo "helm repo pinning gate passed"
