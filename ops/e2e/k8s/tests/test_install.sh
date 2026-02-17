#!/usr/bin/env sh
set -eu
. "$(dirname "$0")/common.sh"
need helm; need kubectl

install_chart
wait_ready

echo "install gate passed"
