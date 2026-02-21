from __future__ import annotations

import argparse
import json
from pathlib import Path

from atlasctl.core.context import RunContext
from atlasctl.commands.policies.runtime.command import run_policies_command


def _ctx(root: Path) -> RunContext:
    return RunContext(
        run_id="r1",
        profile="local",
        repo_root=root,
        evidence_root=root / "artifacts/evidence",
        scripts_artifact_root=root / "artifacts/atlasctl/run/r1",
        run_dir=root / "artifacts/evidence/r1",
        output_format="text",
        network_mode="allow",
        verbose=False,
        quiet=False,
        require_clean_git=False,
        no_network=False,
        git_sha="unknown",
        git_dirty=False,
    )


def test_scan_rust_relaxations_writes_findings(tmp_path: Path) -> None:
    src = tmp_path / "crates/demo/src"
    src.mkdir(parents=True)
    (src / "lib.rs").write_text("#[allow(dead_code)] // ATLAS-EXC-TEST\n", encoding="utf-8")
    ns = argparse.Namespace(
        policies_cmd="scan-rust-relaxations",
        out="artifacts/policy/relaxations-rust.json",
        report="json",
    )
    assert run_policies_command(_ctx(tmp_path), ns) == 0
    payload = json.loads((tmp_path / "artifacts/policy/relaxations-rust.json").read_text(encoding="utf-8"))
    assert payload["findings"]

