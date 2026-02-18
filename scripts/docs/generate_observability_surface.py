#!/usr/bin/env python3
# owner: docs-governance
# purpose: generate docs/_generated/observability-surface.md from observability contract SSOT files.
# stability: public
# called-by: make docs
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "docs/_generated/observability-surface.md"
METRICS = ROOT / "ops/observability/contract/metrics-contract.json"
ALERTS = ROOT / "ops/observability/contract/alerts-contract.json"
DASH = ROOT / "ops/observability/contract/dashboard-panels-contract.json"
LOGS = ROOT / "ops/observability/contract/logs-fields-contract.json"


def read_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def as_list(value: object) -> list[str]:
    if isinstance(value, list):
        return [str(v) for v in value]
    if isinstance(value, dict):
        return [str(k) for k in value.keys()]
    return []


def main() -> int:
    metrics = read_json(METRICS)
    alerts = read_json(ALERTS)
    dash = read_json(DASH)
    logs = read_json(LOGS)

    metric_names = sorted(as_list(metrics.get("required_metrics", [])))
    alert_names = sorted(as_list(alerts.get("required_alerts", [])))
    dashboard_panels = sorted(as_list(dash.get("required_panels", [])))
    log_fields = sorted(as_list(logs.get("required_fields", [])))

    lines = [
        "# Observability Surface",
        "",
        "Generated from observability contract SSOT files:",
        "- `ops/observability/contract/metrics-contract.json`",
        "- `ops/observability/contract/alerts-contract.json`",
        "- `ops/observability/contract/dashboard-panels-contract.json`",
        "- `ops/observability/contract/logs-fields-contract.json`",
        "",
        "## Metrics",
    ]
    lines += [f"- `{name}`" for name in metric_names] or ["- _none_"]
    lines += ["", "## Alerts"]
    lines += [f"- `{name}`" for name in alert_names] or ["- _none_"]
    lines += ["", "## Dashboard Panels"]
    lines += [f"- `{name}`" for name in dashboard_panels] or ["- _none_"]
    lines += ["", "## Log Fields"]
    lines += [f"- `{name}`" for name in log_fields] or ["- _none_"]
    lines += ["", "## Verification", "```bash", "make ops-observability-validate", "```", ""]

    OUT.write_text("\n".join(lines), encoding="utf-8")
    print(f"wrote {OUT.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
