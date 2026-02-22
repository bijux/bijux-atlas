#!/usr/bin/env bash
set -euo pipefail

helm lint ops/k8s/charts/bijux-atlas
helm template atlas ops/k8s/charts/bijux-atlas >/dev/null
