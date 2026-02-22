#!/usr/bin/env bash
set -euo pipefail

mkdir -p artifacts/chart
helm package ops/k8s/charts/bijux-atlas --destination artifacts/chart
