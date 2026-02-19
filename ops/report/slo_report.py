#!/usr/bin/env python3
# Purpose: compute SLO report values from metrics snapshot and SLO config.
# Inputs: --metrics, --slo-config, --out
# Outputs: JSON report with SLI values, error budget remaining, and burn rates.
from __future__ import annotations

import argparse
import datetime as dt
import json
import re
from pathlib import Path
from typing import Any


METRIC_LINE_RE = re.compile(r"^([a-zA-Z_:][a-zA-Z0-9_:]*)(?:\{([^}]*)\})?\s+([-+]?\d+(?:\.\d+)?(?:[eE][-+]?\d+)?)")


def parse_metrics(text: str) -> dict[str, list[dict[str, Any]]]:
    out: dict[str, list[dict[str, Any]]] = {}
    for raw in text.splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        m = METRIC_LINE_RE.match(line)
        if not m:
            continue
        name = m.group(1)
        labels_str = m.group(2) or ""
        value = float(m.group(3))
        labels: dict[str, str] = {}
        if labels_str:
            for part in re.split(r",(?=(?:[^\"]*\"[^\"]*\")*[^\"]*$)", labels_str):
                if "=" not in part:
                    continue
                k, v = part.split("=", 1)
                labels[k.strip()] = v.strip().strip('"')
        out.setdefault(name, []).append({"labels": labels, "value": value})
    return out


def _status_match(actual: str, expected: str) -> bool:
    if expected == "2xx":
        return actual.startswith("2")
    if expected == "5xx":
        return actual.startswith("5")
    if expected == "2xx|5xx":
        return actual.startswith("2") or actual.startswith("5")
    if "|" in expected:
        return actual in expected.split("|")
    return actual == expected


def _label_match(sample_labels: dict[str, str], filters: dict[str, str]) -> bool:
    for key, expected in filters.items():
        if key not in sample_labels:
            return False
        actual = sample_labels[key]
        if key == "status":
            if not _status_match(actual, expected):
                return False
            continue
        if "|" in expected:
            if actual not in expected.split("|"):
                return False
            continue
        if expected in {"*", "<class-pattern>"}:
            continue
        if expected.startswith("^"):
            if re.match(expected, actual) is None:
                return False
            continue
        if actual != expected:
            return False
    return True


def sum_metric(metrics: dict[str, list[dict[str, Any]]], name: str, filters: dict[str, str]) -> float:
    return sum(
        sample["value"]
        for sample in metrics.get(name, [])
        if _label_match(sample["labels"], filters)
    )


def max_metric(metrics: dict[str, list[dict[str, Any]]], name: str, filters: dict[str, str]) -> float | None:
    vals = [
        sample["value"]
        for sample in metrics.get(name, [])
        if _label_match(sample["labels"], filters)
    ]
    return max(vals) if vals else None


def compute_sli_value(metrics: dict[str, list[dict[str, Any]]], sli: dict[str, Any]) -> tuple[float | None, str]:
    metric = sli.get("metric")
    labels = dict(sli.get("labels", {}))
    kind = sli.get("kind")

    if metric == "http_requests_total" and kind in {"availability", "success-rate", "overload"}:
        status = labels.pop("status", None)
        if status == "200":
            good = sum_metric(metrics, metric, {**labels, "status": "200"})
            total = sum_metric(metrics, metric, labels)
            return ((good / total) if total > 0 else None, "ratio")
        good = sum_metric(metrics, metric, {**labels, "status": "2xx"})
        total = sum_metric(metrics, metric, {**labels, "status": "2xx|5xx"})
        return ((good / total) if total > 0 else None, "ratio")

    if metric == "atlas_store_errors_total":
        errors = sum_metric(metrics, metric, labels)
        request_labels = {"class": labels.get("class", "standard|heavy")}
        total = sum_metric(metrics, "http_requests_total", request_labels)
        return ((errors / total) if total > 0 else None, "ratio")

    if metric == "http_request_duration_seconds_bucket":
        # Estimate threshold compliance ratio from histogram buckets.
        le = labels.get("le")
        if not le:
            return (None, "ratio")
        numer = sum_metric(metrics, metric, labels)
        denom_labels = dict(labels)
        denom_labels["le"] = "+Inf"
        denom = sum_metric(metrics, metric, denom_labels)
        return ((numer / denom) if denom > 0 else None, "ratio")

    if kind == "freshness":
        v = max_metric(metrics, metric, labels)
        return (v, "seconds")

    v = max_metric(metrics, metric, labels)
    if v is None:
        return (None, "raw")
    if "latency" in str(sli.get("id", "")) or "latency" in str(kind):
        if metric.endswith("_seconds"):
            return (v * 1000.0, "ms")
    return (v, "raw")


def evaluate_slo(slo: dict[str, Any], sli_value: float | None) -> tuple[bool | None, float | None, float | None]:
    if sli_value is None:
        return (None, None, None)

    threshold = slo.get("threshold")
    target = slo.get("target")

    if isinstance(threshold, dict):
        op = threshold.get("operator")
        val = threshold.get("value")
        if not isinstance(val, (int, float)):
            return (None, None, None)
        if op == "lt":
            return (sli_value < float(val), None, None)
        if op == "gt":
            return (sli_value > float(val), None, None)
        return (None, None, None)

    if not isinstance(target, (int, float)):
        return (None, None, None)

    budget = slo.get("budget")
    observed_error = max(0.0, 1.0 - sli_value)
    if isinstance(budget, (int, float)) and budget > 0:
        budget_remaining = max(0.0, float(budget) - observed_error)
        burn_rate = observed_error / float(budget)
    else:
        budget_remaining = max(0.0, 1.0 - observed_error - float(target))
        burn_rate = None
    return (sli_value >= float(target), budget_remaining, burn_rate)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--metrics", default="artifacts/ops/metrics.prom")
    parser.add_argument("--slo-config", default="configs/ops/slo/slo.v1.json")
    parser.add_argument("--out", default="artifacts/ops/slo/report.json")
    args = parser.parse_args()

    metrics_path = Path(args.metrics)
    slo_path = Path(args.slo_config)
    out_path = Path(args.out)

    metrics = parse_metrics(metrics_path.read_text(encoding="utf-8")) if metrics_path.exists() else {}
    slo_cfg = json.loads(slo_path.read_text(encoding="utf-8"))

    slis_by_id = {sli["id"]: sli for sli in slo_cfg.get("slis", [])}
    sli_rows: list[dict[str, Any]] = []
    for sli_id, sli in slis_by_id.items():
        value, unit = compute_sli_value(metrics, sli)
        sli_rows.append(
            {
                "id": sli_id,
                "kind": sli.get("kind"),
                "metric": sli.get("metric"),
                "value": value,
                "unit": unit,
                "window": sli.get("window"),
                "status": "measured" if value is not None else "insufficient_data",
            }
        )

    sli_value_map = {row["id"]: row["value"] for row in sli_rows}
    slo_rows: list[dict[str, Any]] = []
    for slo in slo_cfg.get("slos", []):
        sid = slo.get("sli")
        value = sli_value_map.get(sid)
        compliant, budget_remaining, burn_rate = evaluate_slo(slo, value)
        slo_rows.append(
            {
                "id": slo.get("id"),
                "sli": sid,
                "value": value,
                "target": slo.get("target"),
                "window": slo.get("window"),
                "threshold": slo.get("threshold"),
                "compliant": compliant,
                "error_budget_remaining": budget_remaining,
                "burn_rate": burn_rate,
            }
        )

    compliant = sum(1 for row in slo_rows if row["compliant"] is True)
    violated = sum(1 for row in slo_rows if row["compliant"] is False)
    unknown = sum(1 for row in slo_rows if row["compliant"] is None)
    total = len(slo_rows)
    summary = {
        "total_slos": total,
        "compliant_slos": compliant,
        "violated_slos": violated,
        "unknown_slos": unknown,
        "compliance_ratio": (compliant / total) if total else 0.0,
    }

    payload = {
        "generated_at": dt.datetime.now(tz=dt.timezone.utc).isoformat(),
        "slo_version": slo_cfg.get("version", "v1"),
        "metrics_source": str(metrics_path),
        "slis": sli_rows,
        "slos": slo_rows,
        "summary": summary,
    }

    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(out_path)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
