from __future__ import annotations

import json
from pathlib import Path

from atlasctl.commands.configs.command import _generate_outputs


def _repo_root() -> Path:
    here = Path(__file__).resolve()
    for parent in here.parents:
        if (parent / "packages/atlasctl/src/atlasctl").exists():
            return parent
    raise AssertionError("unable to locate repository root")


def test_generate_outputs_includes_public_command_artifacts() -> None:
    outputs = _generate_outputs(_repo_root(), write=False)
    assert "docs/_generated/public-commands.json" in outputs
    assert "docs/_generated/public-commands.md" in outputs
    payload = json.loads(outputs["docs/_generated/public-commands.json"])
    assert payload["kind"] == "public-command-list"
    assert isinstance(payload["commands"], list) and payload["commands"]

