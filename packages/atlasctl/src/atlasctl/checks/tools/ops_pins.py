from __future__ import annotations

import json
from pathlib import Path


def check_ops_pins_update_is_deterministic(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    pins = repo_root / "configs/ops/pins.json"
    if not pins.exists():
        return 1, ["missing configs/ops/pins.json"]
    before = pins.read_text(encoding="utf-8")
    try:
        payload = json.loads(before)
    except json.JSONDecodeError as exc:
        return 1, [f"configs/ops/pins.json invalid json: {exc}"]
    canonical = json.dumps(payload, indent=2, sort_keys=True) + "\n"
    if before != canonical:
        errors.append("configs/ops/pins.json must be canonical sorted JSON")
    required = {"schema_version", "contract_version", "tools", "images", "helm", "datasets", "policy"}
    missing = sorted(required - set(payload.keys()))
    if missing:
        errors.append(f"configs/ops/pins.json missing required keys: {', '.join(missing)}")
    return (0 if not errors else 1), errors


def check_ops_pins_no_unpinned_versions(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    pins = repo_root / "configs/ops/pins.json"
    if not pins.exists():
        return 1, ["missing configs/ops/pins.json"]
    payload = json.loads(pins.read_text(encoding="utf-8"))
    tools = payload.get("tools", {})
    if not isinstance(tools, dict):
        errors.append("configs/ops/pins.json `tools` must be an object")
    else:
        for name, spec in sorted(tools.items()):
            if not isinstance(spec, dict):
                errors.append(f"configs/ops/pins.json `tools.{name}` must be an object")
                continue
            version = str(spec.get("version", "")).strip().lower()
            if not version:
                errors.append(f"configs/ops/pins.json `tools.{name}` missing version")
                continue
            if version in {"latest", "main", "master"}:
                errors.append(f"configs/ops/pins.json `tools.{name}` uses floating version `{version}`")
    images = payload.get("images", {})
    if not isinstance(images, dict):
        errors.append("configs/ops/pins.json `images` must be an object")
    else:
        for name, spec in sorted(images.items()):
            if not isinstance(spec, dict):
                errors.append(f"configs/ops/pins.json `images.{name}` must be an object")
                continue
            ref = str(spec.get("ref", "")).strip().lower()
            if not ref:
                errors.append(f"configs/ops/pins.json `images.{name}` missing ref")
                continue
            if ref.endswith(":latest") or ref in {"latest", "main", "master"}:
                errors.append(f"configs/ops/pins.json `images.{name}` uses floating ref `{ref}`")
    helm = payload.get("helm", {})
    if not isinstance(helm, dict):
        errors.append("configs/ops/pins.json `helm` must be an object")
    else:
        deps = helm.get("chart_dependencies", [])
        if deps is None:
            deps = []
        if not isinstance(deps, list):
            errors.append("configs/ops/pins.json `helm.chart_dependencies` must be a list")
        else:
            for idx, dep in enumerate(deps):
                if not isinstance(dep, dict):
                    errors.append(f"configs/ops/pins.json `helm.chart_dependencies[{idx}]` must be an object")
                    continue
                version = str(dep.get("version", "")).strip().lower()
                if not version:
                    errors.append(f"configs/ops/pins.json `helm.chart_dependencies[{idx}]` missing version")
                    continue
                if version in {"latest", "main", "master"}:
                    errors.append(f"configs/ops/pins.json `helm.chart_dependencies[{idx}]` uses floating version `{version}`")
    return (0 if not errors else 1), errors

