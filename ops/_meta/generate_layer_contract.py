#!/usr/bin/env python3
from __future__ import annotations

import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "ops" / "_meta" / "layer-contract.json"
VALUES = ROOT / "ops" / "k8s" / "charts" / "bijux-atlas" / "values.yaml"
CHART = ROOT / "ops" / "k8s" / "charts" / "bijux-atlas" / "Chart.yaml"
STACK_FILES = {
    "minio": ROOT / "ops" / "stack" / "minio" / "minio.yaml",
    "prometheus": ROOT / "ops" / "stack" / "prometheus" / "prometheus.yaml",
    "otel": ROOT / "ops" / "stack" / "otel" / "otel-collector.yaml",
    "grafana": ROOT / "ops" / "stack" / "grafana" / "grafana.yaml",
    "redis": ROOT / "ops" / "stack" / "redis" / "redis.yaml",
}


def _find_scalar(text: str, key: str) -> str:
    m = re.search(rf"^\s*{re.escape(key)}:\s*\"?([^\"\n]+)\"?\s*$", text, flags=re.M)
    return m.group(1).strip() if m else ""


def _find_namespace(text: str) -> str:
    m = re.search(r"^\s*namespace:\s*([a-z0-9-]+)\s*$", text, flags=re.M)
    return m.group(1) if m else "atlas-e2e"


def _service_ports(text: str) -> dict[str, int]:
    ports: dict[str, int] = {}
    for name, num in re.findall(r"-\s+name:\s*([a-z0-9-]+)\s*\n\s*port:\s*([0-9]+)", text, flags=re.I):
        ports[name] = int(num)
    if not ports:
        for num in re.findall(r"\bport:\s*([0-9]+)", text):
            ports["service"] = int(num)
            break
    return ports


def _deployment_ports(text: str) -> dict[str, int]:
    ports: dict[str, int] = {}
    for name, num in re.findall(r"-\s+name:\s*([a-z0-9-]+)\s*\n\s*containerPort:\s*([0-9]+)", text, flags=re.I):
        ports[name] = int(num)
    if not ports:
        for num in re.findall(r"containerPort:\s*([0-9]+)", text):
            ports["container"] = int(num)
            break
    return ports


def _metadata_name(text: str, kind: str) -> str:
    pattern = rf"kind:\s*{re.escape(kind)}.*?metadata:\s*\n\s*name:\s*([a-z0-9-]+)"
    m = re.search(pattern, text, flags=re.S | re.I)
    return m.group(1) if m else ""


def main() -> int:
    values = VALUES.read_text(encoding="utf-8")
    chart = CHART.read_text(encoding="utf-8")

    ns = "atlas-e2e"
    stack_texts = {name: path.read_text(encoding="utf-8") for name, path in STACK_FILES.items()}
    for text in stack_texts.values():
        ns = _find_namespace(text)
        if ns:
            break

    atlas_port = int(_find_scalar(values, "port") or "8080")
    chart_name = _find_scalar(chart, "name") or "bijux-atlas"
    chart_version = _find_scalar(chart, "version") or "0.0.0"
    app_version = _find_scalar(chart, "appVersion") or "0.0.0"

    services = {
        "atlas": {
            "service_name": "atlas-e2e-bijux-atlas",
            "selector": {
                "app.kubernetes.io/instance": "atlas-e2e",
                "app.kubernetes.io/name": "bijux-atlas",
            },
        }
    }
    for name, text in stack_texts.items():
        services[name] = {
            "service_name": _metadata_name(text, "Service") or name,
            "selector": {"app": name if name != "otel" else "otel-collector"},
        }

    ports = {
        "atlas": {"service": atlas_port, "container": atlas_port},
    }
    for name, text in stack_texts.items():
        svc = _service_ports(text)
        dep = _deployment_ports(text)
        if name == "otel":
            ports[name] = {
                "grpc": svc.get("grpc", dep.get("grpc", 4317)),
                "http": svc.get("http", dep.get("http", 4318)),
            }
        elif name == "minio":
            ports[name] = {
                "api": svc.get("api", dep.get("api", 9000)),
                "console": svc.get("console", dep.get("console", 9001)),
            }
        else:
            ports[name] = {
                "service": next(iter(svc.values()), 0),
                "container": next(iter(dep.values()), next(iter(svc.values()), 0)),
            }

    data = {
        "contract_version": "1.0.0",
        "compatibility": {
            "policy": "Minor/patch updates are backward-compatible; major updates may remove or rename fields.",
            "current_major": 1,
            "notes": [
                "Consumers must ignore unknown fields.",
                "Required keys are validated by ops/_schemas/meta/layer-contract.schema.json.",
            ],
        },
        "namespaces": {"stack": ns, "k8s": ns, "e2e": ns},
        "services": services,
        "ports": ports,
        "labels": {
            "required": ["app.kubernetes.io/name", "app.kubernetes.io/instance"],
            "required_annotations": [],
        },
        "release_metadata": {
            "required_fields": ["release_name", "namespace", "chart_name", "chart_version", "app_version"],
            "defaults": {
                "release_name": "atlas-e2e",
                "namespace": ns,
                "chart_name": chart_name,
                "chart_version": chart_version,
                "app_version": app_version,
            },
        },
    }

    OUT.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"wrote {OUT.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
