from __future__ import annotations

import json
import os
from pathlib import Path

from atlasctl.core.runtime.paths import write_text_file


def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for parent in cur.parents:
        if all((parent / marker).exists() for marker in ("makefiles", "packages", "configs", "ops")):
            return parent
    raise RuntimeError("unable to resolve repo root")


def _layer_contract(root: Path) -> dict[str, object]:
    return json.loads((root / "ops/_meta/layer-contract.json").read_text(encoding="utf-8"))


def port_atlas(root: Path | None = None) -> int:
    root = root or _repo_root()
    fallback = int(_layer_contract(root)["ports"]["atlas"]["service"])  # type: ignore[index]
    return int(os.environ.get("ATLAS_PORT", str(fallback)))


def port_prometheus(root: Path | None = None) -> int:
    root = root or _repo_root()
    fallback = int(_layer_contract(root)["ports"]["prometheus"]["service"])  # type: ignore[index]
    return int(os.environ.get("ATLAS_PROM_PORT", str(fallback)))


def port_grafana(root: Path | None = None) -> int:
    root = root or _repo_root()
    fallback = int(_layer_contract(root)["ports"]["grafana"]["service"])  # type: ignore[index]
    return int(os.environ.get("ATLAS_GRAFANA_PORT", str(fallback)))


def url_atlas(root: Path | None = None) -> str:
    root = root or _repo_root()
    return os.environ.get("ATLAS_BASE_URL", f"http://127.0.0.1:{port_atlas(root)}")


def url_grafana(root: Path | None = None) -> str:
    root = root or _repo_root()
    return os.environ.get("ATLAS_GRAFANA_URL", f"http://127.0.0.1:{port_grafana(root)}")


def url_prometheus(root: Path | None = None) -> str:
    root = root or _repo_root()
    return os.environ.get("ATLAS_PROM_URL", f"http://127.0.0.1:{port_prometheus(root)}")


def publish_ports_json(out_path: Path, root: Path | None = None) -> Path:
    root = root or _repo_root()
    payload = {
        "atlas": {"port": port_atlas(root), "url": url_atlas(root)},
        "prometheus": {"port": port_prometheus(root), "url": url_prometheus(root)},
        "grafana": {"port": port_grafana(root), "url": url_grafana(root)},
    }
    write_text_file(out_path, json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return out_path

