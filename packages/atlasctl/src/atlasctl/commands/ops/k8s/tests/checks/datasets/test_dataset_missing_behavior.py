#!/usr/bin/env python3
from __future__ import annotations
from pathlib import Path
from ._shell_common import run_k8s_test_shell

def main() -> int:
    return run_k8s_test_shell(
        """
setup_test_traps
need curl
install_chart
wait_ready
with_port_forward 18080
code="$(curl -s -o /tmp/missing_dataset_resp.json -w '%{http_code}' "$BASE_URL/v1/genes?release=999&species=homo_sapiens&assembly=GRCh38&gene_id=GENE1" || true)"
[ "$code" = "404" ] || { echo "expected 404 for missing dataset, got $code" >&2; exit 1; }
if ! curl -fsS "$BASE_URL/metrics" | grep -Eq 'bijux_http_requests_total|bijux_dataset_misses'; then
  pod="$(pod_name)"
  kubectl -n "$NS" logs "$pod" --tail=200 | grep -Ei 'not found|dataset' >/dev/null
fi
echo "missing dataset behavior gate passed"
        """,
        Path(__file__),
    )

if __name__ == "__main__":
    raise SystemExit(main())
