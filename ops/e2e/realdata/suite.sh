#!/usr/bin/env bash
set -euo pipefail
DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
SUITE="${1:-}"
if [ "$SUITE" = "--suite" ]; then
  SUITE="${2:-full}"
fi
SUITE="${SUITE:-full}"
case "$SUITE" in
  full)
    exec "$DIR/run_all.sh"
    ;;
  *)
    echo "unknown realdata suite: $SUITE" >&2
    exit 2
    ;;
esac
