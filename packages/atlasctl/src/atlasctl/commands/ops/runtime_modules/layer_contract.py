from __future__ import annotations

import json
from pathlib import Path
from typing import Any


def load_layer_contract(repo_root: Path) -> dict[str, Any]:
    return json.loads((repo_root / "ops/_meta/layer-contract.json").read_text(encoding="utf-8"))


def _walk(obj: Any, parts: list[str], key: str) -> Any:
    cur = obj
    for part in parts:
        if isinstance(cur, dict) and part in cur:
            cur = cur[part]
        else:
            raise KeyError(f"missing key: {key}")
    return cur


def get_layer_contract_value(contract: dict[str, Any], key: str, default: Any = None) -> Any:
    try:
        return _walk(contract, key.split("."), key)
    except KeyError:
        return default


def ns_e2e(contract: dict[str, Any], default: str = "atlas-e2e") -> str:
    return str(get_layer_contract_value(contract, "namespaces.e2e", default))


def ns_k8s(contract: dict[str, Any], default: str = "atlas-e2e") -> str:
    return str(get_layer_contract_value(contract, "namespaces.k8s", default))


def service_atlas(contract: dict[str, Any], default: str = "bijux-atlas") -> str:
    return str(get_layer_contract_value(contract, "atlas_services.atlas", get_layer_contract_value(contract, "services.atlas.service_name", default)))


def release_default(contract: dict[str, Any], default: str = "atlas-e2e") -> str:
    return str(get_layer_contract_value(contract, "release_metadata.defaults.release_name", default))

