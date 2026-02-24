from __future__ import annotations

import json
import re
from pathlib import Path

from atlasctl.core.schema.yaml_utils import load_yaml


def _read_json(path: Path) -> dict[str, object]:
    return json.loads(path.read_text(encoding="utf-8"))


def _read_yaml_map(path: Path) -> dict[str, object]:
    payload = load_yaml(path)
    return payload if isinstance(payload, dict) else {}


def check_ops_workflow_evidence_path_policy(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    root = repo_root / "packages/atlasctl/src/atlasctl/ops/workflows"
    if not root.exists():
        return 1, ["missing atlasctl ops workflows package"]
    for path in sorted(root.glob("*.py")):
        text = path.read_text(encoding="utf-8")
        rel = path.relative_to(repo_root).as_posix()
        if "ops_evidence_dir(" in text:
            errors.append(f"{rel}: legacy ops_evidence_dir usage forbidden in first-class ops workflows")
        if "artifacts/evidence" in text:
            errors.append(f"{rel}: artifacts/evidence path forbidden; use artifacts/runs/<run_id>/ops/... helper")
    helper = root / "paths.py"
    if not helper.exists():
        errors.append("packages/atlasctl/src/atlasctl/ops/workflows/paths.py missing")
    return (0 if not errors else 1), (["ops workflow evidence path policy passed"] if not errors else errors)


def check_ops_helm_render_inventory_drift(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    chart = _read_yaml_map(repo_root / "ops/k8s/charts/bijux-atlas/Chart.yaml")
    layers = _read_json(repo_root / "ops/inventory/layers.json")
    values_schema = repo_root / "ops/k8s/charts/bijux-atlas/values.schema.json"
    if not values_schema.exists():
        errors.append("missing ops/k8s/charts/bijux-atlas/values.schema.json")
    else:
        try:
            json.loads(values_schema.read_text(encoding="utf-8"))
        except Exception as exc:
            errors.append(f"invalid values.schema.json: {exc}")
    defaults = (((layers.get("release_metadata") or {}) if isinstance(layers, dict) else {}).get("defaults") or {})
    if isinstance(defaults, dict):
        if str(defaults.get("chart_name", "")) != str(chart.get("name", "")):
            errors.append("layers.release_metadata.defaults.chart_name must match Chart.yaml name")
        if str(defaults.get("chart_version", "")) != str(chart.get("version", "")):
            errors.append("layers.release_metadata.defaults.chart_version must match Chart.yaml version")
        if str(defaults.get("app_version", "")) != str(chart.get("appVersion", "")):
            errors.append("layers.release_metadata.defaults.app_version must match Chart.yaml appVersion")
    return (0 if not errors else 1), (["ops helm inventory drift check passed"] if not errors else errors)


def check_ops_images_pinned_to_digests_policy(repo_root: Path) -> tuple[int, list[str]]:
    toolchain = _read_yaml_map(repo_root / "ops/inventory/toolchain.yaml")
    policy = _read_json(repo_root / "configs/policy/ops-image-digest-policy.json")
    images = toolchain.get("images", {})
    allow = set(policy.get("allow_non_digest_image_keys", [])) if isinstance(policy.get("allow_non_digest_image_keys"), list) else set()
    errors: list[str] = []
    if not isinstance(images, dict):
        return 1, ["ops/inventory/toolchain.yaml images must be a mapping"]
    for key, value in sorted(images.items()):
        image_ref = str(value)
        if "@sha256:" in image_ref:
            continue
        if str(key) in allow:
            continue
        errors.append(f"image `{key}` must be pinned to digest or allowlisted (got `{image_ref}`)")
    return (0 if not errors else 1), (["ops image digest policy passed"] if not errors else errors)


def check_ops_helm_versions_pinned(repo_root: Path) -> tuple[int, list[str]]:
    chart = _read_yaml_map(repo_root / "ops/k8s/charts/bijux-atlas/Chart.yaml")
    helm_pins = _read_json(repo_root / "configs/ops/pins/helm.json")
    errors: list[str] = []
    for field in ("version", "appVersion"):
        value = str(chart.get(field, ""))
        if not value:
            errors.append(f"Chart.yaml missing {field}")
        if any(x in value for x in ("*", "x", "latest", ">=", "<=", "~", "^")):
            errors.append(f"Chart.yaml {field} looks floating: {value}")
    if not isinstance(helm_pins.get("helm", {}), dict):
        errors.append("configs/ops/pins/helm.json must contain helm object")
    return (0 if not errors else 1), (["ops helm versions pinned"] if not errors else errors)


def check_ops_namespaces_inventory_manifest_agreement(repo_root: Path) -> tuple[int, list[str]]:
    layers = _read_json(repo_root / "ops/inventory/layers.json")
    ns_cfg = _read_json(repo_root / "configs/ops/namespaces.json")
    chart_values = _read_yaml_map(repo_root / "ops/k8s/charts/bijux-atlas/values.yaml")
    errors: list[str] = []
    layers_ns = layers.get("namespaces", {})
    cfg_ns = ns_cfg.get("namespaces", {})
    if not isinstance(layers_ns, dict) or not isinstance(cfg_ns, dict):
        return 1, ["namespace maps missing in layers.json or configs/ops/namespaces.json"]
    for key, value in sorted(layers_ns.items()):
        if cfg_ns.get(key) != value:
            errors.append(f"namespace mismatch for `{key}`: layers={value} configs={cfg_ns.get(key)}")
    release_ns = (((layers.get("release_metadata") or {}) if isinstance(layers, dict) else {}).get("defaults") or {}).get("namespace")
    values_ns = chart_values.get("namespace")
    if release_ns and values_ns and str(release_ns) != str(values_ns):
        errors.append(f"chart values namespace `{values_ns}` must match layers release namespace `{release_ns}`")
    return (0 if not errors else 1), (["ops namespaces agree"] if not errors else errors)


def check_ops_ports_and_surfaces_contract(repo_root: Path) -> tuple[int, list[str]]:
    layers = _read_json(repo_root / "ops/inventory/layers.json")
    ports_cfg = _read_json(repo_root / "configs/ops/ports.json")
    surfaces = _read_json(repo_root / "ops/inventory/surfaces.json")
    errors: list[str] = []
    layers_ports = layers.get("ports", {})
    cfg_ports = ports_cfg.get("ports", {})
    if not isinstance(layers_ports, dict) or not isinstance(cfg_ports, dict):
        return 1, ["ports maps missing in layers.json or configs/ops/ports.json"]
    for service, port_map in sorted(layers_ports.items()):
        if service not in cfg_ports:
            errors.append(f"layer port service `{service}` missing in configs/ops/ports.json")
            continue
        if cfg_ports.get(service) != port_map:
            errors.append(f"port mismatch for `{service}` between layers.json and configs/ops/ports.json")
    entrypoints = surfaces.get("entrypoints", [])
    if not isinstance(entrypoints, list) or not entrypoints:
        errors.append("ops/inventory/surfaces.json must contain non-empty entrypoints")
    docs_stack = repo_root / "docs/operations/e2e/stack.md"
    if docs_stack.exists():
        stack_text = docs_stack.read_text(encoding="utf-8")
        for service in sorted(layers_ports.keys()):
            if service == "atlas":
                continue
            if service not in stack_text:
                errors.append(f"docs/operations/e2e/stack.md missing service/port surface reference `{service}`")
    return (0 if not errors else 1), (["ops ports/surfaces contract passed"] if not errors else errors)


def check_ops_observability_assets_for_public_surfaces(repo_root: Path) -> tuple[int, list[str]]:
    layers = _read_json(repo_root / "ops/inventory/layers.json")
    services = layers.get("services", {})
    errors: list[str] = []
    if not isinstance(services, dict):
        return 1, ["ops/inventory/layers.json services must be a mapping"]
    candidate_files = []
    for root in (repo_root / "ops/observe", repo_root / "ops/obs"):
        if root.exists():
            candidate_files.extend(p for p in root.rglob("*") if p.is_file())
    corpus = "\n".join(p.read_text(encoding="utf-8", errors="ignore") for p in candidate_files)
    for service in sorted(services.keys()):
        if re.search(rf"\b{re.escape(service)}\b", corpus) is None:
            errors.append(f"observability assets missing coverage/reference for public service `{service}`")
    return (0 if not errors else 1), (["ops observability assets cover public surfaces"] if not errors else errors)


def check_ops_load_thresholds_exist_for_every_suite(repo_root: Path) -> tuple[int, list[str]]:
    suites = _read_json(repo_root / "ops/load/suites/suites.json")
    payload_suites = suites.get("suites", [])
    errors: list[str] = []
    if not isinstance(payload_suites, list):
        return 1, ["ops/load/suites/suites.json suites must be a list"]
    thresholds_dir = repo_root / "ops/load/thresholds"
    for row in payload_suites:
        if not isinstance(row, dict):
            continue
        name = str(row.get("name", "")).strip()
        if not name:
            continue
        threshold_file = thresholds_dir / f"{name}.thresholds.json"
        if not threshold_file.exists():
            errors.append(f"missing threshold file for load suite `{name}`: {threshold_file.relative_to(repo_root).as_posix()}")
            continue
        try:
            threshold_payload = _read_json(threshold_file)
        except Exception as exc:
            errors.append(f"invalid threshold file for `{name}`: {exc}")
            continue
        if str(threshold_payload.get("suite", "")) != name:
            errors.append(f"{threshold_file.relative_to(repo_root).as_posix()}: suite field must equal `{name}`")
    return (0 if not errors else 1), (["ops load thresholds coverage passed"] if not errors else errors)
