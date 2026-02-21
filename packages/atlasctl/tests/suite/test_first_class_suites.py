from __future__ import annotations

import json
from pathlib import Path

from helpers import run_atlasctl

ROOT = Path(__file__).resolve().parents[1]


def _golden(name: str) -> str:
    return (ROOT / "goldens" / name).read_text(encoding="utf-8").strip()


def test_first_class_suite_list_docs() -> None:
    proc = run_atlasctl("--quiet", "suite", "docs", "--list", "--json")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["suite"] == "docs"
    assert payload["required_env"] == ["PYTHONPATH"]
    assert payload["total_count"] >= 1
    assert all(item.startswith("docs.") for item in payload["check_ids"])


def test_run_suite_alias_docs_list() -> None:
    proc = run_atlasctl("--quiet", "run", "suite", "docs", "--list", "--json")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["suite"] == "docs"
    assert payload["total_count"] >= 1


def test_first_class_suite_check_passes() -> None:
    proc = run_atlasctl("--quiet", "suite", "check", "--json")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["status"] == "ok"


def test_suite_inventory_golden() -> None:
    proc = run_atlasctl("--quiet", "suite", "list", "--json")
    assert proc.returncode == 0, proc.stderr
    assert proc.stdout.strip() == _golden("suite-inventory.json.golden")
