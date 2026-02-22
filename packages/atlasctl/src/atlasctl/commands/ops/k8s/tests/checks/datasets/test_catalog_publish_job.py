#!/usr/bin/env python3
from __future__ import annotations
from pathlib import Path
from ._shell_common import run_k8s_test_shell

def main() -> int:
    return run_k8s_test_shell(
        """
setup_test_traps
need helm kubectl curl
TMP_VALUES="$(mktemp)"
cat > "$TMP_VALUES" <<YAML
catalogPublishJob:
  enabled: true
YAML
install_chart -f "$TMP_VALUES"
wait_kubectl_condition job "$SERVICE_NAME-catalog-publish" complete 300s
with_port_forward 18080
wait_for_http "$BASE_URL/metrics" 200 60
curl -fsS "$BASE_URL/metrics" | grep -q "bijux_catalog_epoch"
echo "catalog publish job gate passed"
        """,
        Path(__file__),
    )

if __name__ == "__main__":
    raise SystemExit(main())
