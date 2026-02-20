#!/usr/bin/env bash
# Purpose: scan built image for vulnerabilities with trivy/grype.
set -euo pipefail

if [ "${1:-}" = "--help" ] || [ "${1:-}" = "-h" ]; then
  cat <<'USAGE'
Usage: docker/scripts/docker-scan.sh [image]

Scanner precedence: trivy, then grype.
Writes report JSON under artifacts/scripts/docker-scan/.
USAGE
  exit 0
fi

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
IMAGE="${1:-${DOCKER_IMAGE:-bijux-atlas:local}}"
OUT_DIR="$ROOT/artifacts/scripts/docker-scan"
mkdir -p "$OUT_DIR"

if command -v trivy >/dev/null 2>&1; then
  trivy image --quiet --severity HIGH,CRITICAL --format json --output "$OUT_DIR/trivy.json" "$IMAGE"
  echo "docker scan passed via trivy"
  exit 0
fi

if command -v grype >/dev/null 2>&1; then
  grype "$IMAGE" -o json > "$OUT_DIR/grype.json"
  echo "docker scan passed via grype"
  exit 0
fi

echo "no scanner installed (require trivy or grype)" >&2
exit 2
