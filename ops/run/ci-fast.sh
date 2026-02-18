#!/usr/bin/env bash
set -euo pipefail
exec make ops-up ops-deploy ops-smoke ops-k8s-tests ops-load-smoke ops-observability-validate
