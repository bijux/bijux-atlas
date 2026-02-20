from __future__ import annotations

import json
from pathlib import Path

from bijux_atlas_scripts.check.native import (
    check_duplicate_script_names,
    check_script_ownership,
)


def test_check_duplicate_script_names_detects_dash_underscore_conflict(tmp_path: Path) -> None:
    scripts = tmp_path / "scripts"
    scripts.mkdir()
    (scripts / "a_b.py").write_text("print('x')\n", encoding="utf-8")
    (scripts / "a-b.sh").write_text("#!/usr/bin/env sh\n", encoding="utf-8")
    code, errors = check_duplicate_script_names(tmp_path)
    assert code == 1
    assert errors


def test_check_script_ownership_passes_for_mapped_paths(tmp_path: Path) -> None:
    meta = tmp_path / "scripts/areas/_meta"
    meta.mkdir(parents=True)
    (tmp_path / "scripts/bin").mkdir(parents=True)
    (tmp_path / "scripts/bin/tool.sh").write_text("#!/usr/bin/env sh\n", encoding="utf-8")
    ownership = {"areas": ["scripts/bin", "scripts/areas/_meta"]}
    (meta / "ownership.json").write_text(json.dumps(ownership), encoding="utf-8")
    code, errors = check_script_ownership(tmp_path)
    assert code == 0
    assert errors == []
