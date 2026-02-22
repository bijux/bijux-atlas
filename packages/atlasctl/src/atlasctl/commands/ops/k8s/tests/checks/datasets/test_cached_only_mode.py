#!/usr/bin/env python3
from __future__ import annotations
from pathlib import Path
from ._shell_common import run_k8s_test_shell

def main() -> int:
    return run_k8s_test_shell(
        """
setup_test_traps
need helm; need curl
TMP_VALUES="$(mktemp)"
cat > "$TMP_VALUES" <<YAML
server:
  cachedOnlyMode: true
catalog:
  endpoint: "http://non-existent-catalog.invalid/catalog.json"
YAML
install_chart -f "$TMP_VALUES"
wait_ready
with_port_forward 18080
wait_for_http "$BASE_URL/healthz" 200 60
echo "cached-only mode gate passed"
        """,
        Path(__file__),
    )

if __name__ == "__main__":
    raise SystemExit(main())
