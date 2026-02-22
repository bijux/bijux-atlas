#!/usr/bin/env python3
from __future__ import annotations
from pathlib import Path
from ._shell_common import run_k8s_test_shell

def main() -> int:
    return run_k8s_test_shell(
        """
setup_test_traps
need curl
OFFLINE="$ROOT/ops/k8s/values/offline.yaml"
install_chart -f "$OFFLINE"
wait_ready
with_port_forward 18080
tmp_bad_store="$(mktemp)"
cat >"$tmp_bad_store" <<YAML
store:
  endpoint: http://unreachable-store.invalid:9000
catalog:
  cacheOnlyMode: true
YAML
helm upgrade --install "$RELEASE" "$CHART" -n "$NS" --create-namespace -f "$VALUES" -f "$OFFLINE" -f "$tmp_bad_store" >/dev/null
wait_ready
curl -fsS "$BASE_URL/readyz" >/dev/null
rm -f "$tmp_bad_store"
echo "offline profile gate passed"
        """,
        Path(__file__),
    )

if __name__ == "__main__":
    raise SystemExit(main())
