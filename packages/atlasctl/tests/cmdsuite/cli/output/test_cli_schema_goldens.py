from __future__ import annotations

import json

from atlasctl.contracts.validate import validate
from tests.helpers import golden_text, run_atlasctl


def _golden(name: str) -> str:
    return golden_text(name)


def test_surface_json_matches_schema_and_golden() -> None:
    proc = run_atlasctl("--quiet", "surface", "--json")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    validate("atlasctl.surface.v1", payload)
    assert proc.stdout.strip() == _golden("surface.json.golden")


def test_check_list_json_matches_schema_and_golden() -> None:
    proc = run_atlasctl("--quiet", "--json", "check", "list")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    validate("atlasctl.check-list.v1", payload)
    assert proc.stdout.strip() == _golden("check-list.json.golden")


def test_check_list_json_category_lint_matches_golden() -> None:
    proc = run_atlasctl("--quiet", "--json", "check", "list", "--category", "lint")
    assert proc.returncode == 0, proc.stderr
    assert proc.stdout.strip() == _golden("check/check-list-category-lint.json.golden")


def test_check_list_json_domain_ops_matches_golden() -> None:
    proc = run_atlasctl("--quiet", "--json", "check", "list", "--domain", "ops")
    assert proc.returncode == 0, proc.stderr
    assert proc.stdout.strip() == _golden("check/check-list-domain-ops.json.golden")


def test_checks_list_taxonomy_json_matches_schema_and_golden() -> None:
    proc = run_atlasctl("--quiet", "--json", "checks", "list")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    validate("atlasctl.check-taxonomy.v1", payload)
    assert proc.stdout.strip() == _golden("check/checks-list-taxonomy.json.golden")


def test_checks_tree_json_matches_golden() -> None:
    proc = run_atlasctl("--quiet", "--json", "checks", "tree")
    assert proc.returncode == 0, proc.stderr
    assert proc.stdout.strip() == _golden("check/checks-tree.json.golden")


def test_check_explain_json_matches_golden() -> None:
    proc = run_atlasctl("--quiet", "--json", "check", "explain", "checks.import_cycles")
    assert proc.returncode == 0, proc.stderr
    assert proc.stdout.strip() == _golden("check/check-explain-import-cycles.json.golden")
