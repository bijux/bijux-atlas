from __future__ import annotations

from dataclasses import dataclass

from atlasctl.core.schema.yaml_utils import load_yaml


@dataclass(frozen=True)
class ToolchainInventory:
    schema_version: int
    tools: dict[str, str]
    images: dict[str, str]
    source_path: str

    @classmethod
    def load(cls, repo_root) -> "ToolchainInventory":  # noqa: ANN001
        path = repo_root / "ops" / "inventory" / "toolchain.yaml"
        payload = load_yaml(path) or {}
        if not isinstance(payload, dict):
            payload = {}
        tools = payload.get("tools", {})
        images = payload.get("images", {})
        return cls(
            schema_version=int(payload.get("schema_version", 1) or 1),
            tools={str(k): str(v) for k, v in (tools.items() if isinstance(tools, dict) else [])},
            images={str(k): str(v) for k, v in (images.items() if isinstance(images, dict) else [])},
            source_path="ops/inventory/toolchain.yaml",
        )
