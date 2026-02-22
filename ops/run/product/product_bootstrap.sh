#!/usr/bin/env bash
set -euo pipefail

python3 --version
command -v pip >/dev/null 2>&1 || { echo "missing pip" >&2; exit 1; }
python3 -m pip install -r configs/docs/requirements.txt >/dev/null
command -v k6 >/dev/null 2>&1 || echo "k6 not found (optional for non-perf workflows)"
command -v kind >/dev/null 2>&1 || echo "kind not found (required for k8s e2e)"
command -v kubectl >/dev/null 2>&1 || echo "kubectl not found (required for k8s e2e)"
