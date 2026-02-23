from __future__ import annotations

import shutil

from atlasctl.ops.models import ToolchainInventory


def env_doctor(ctx) -> dict[str, object]:  # noqa: ANN001
    inventory = ToolchainInventory.load(ctx.repo_root)
    tools = []
    for name, pinned in sorted(inventory.tools.items()):
        tools.append(
            {
                "name": name,
                "pinned": pinned,
                "present": shutil.which(name) is not None,
            }
        )
    payload = {
        "schema_name": "atlasctl.ops-env-doctor.v1",
        "kind": "ops-env-doctor",
        "run_id": ctx.run_id,
        "status": "ok",
        "toolchain_source": inventory.source_path,
        "toolchain_schema_version": inventory.schema_version,
        "tools": tools,
        "images": [{"name": k, "ref": v} for k, v in sorted(inventory.images.items())],
    }
    return payload
