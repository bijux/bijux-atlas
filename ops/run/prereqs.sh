#!/usr/bin/env bash
set -euo pipefail
for c in docker kind kubectl helm k6 python3; do
  command -v "$c" >/dev/null 2>&1 || { echo "missing: $c" >&2; exit 1; }
done
python3 --version
kubectl version --client >/dev/null
helm version --short >/dev/null
kind version >/dev/null
k6 version >/dev/null
python3 ./scripts/layout/check_tool_versions.py kind kubectl helm k6 >/dev/null
