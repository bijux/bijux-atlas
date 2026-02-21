from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path

from atlasctl.commands.docs.legacy import _mkdocs_nav_file_refs

ROOT = Path(__file__).resolve().parents[3]


def _run_cli(*args: str) -> subprocess.CompletedProcess[str]:
    env = {"PYTHONPATH": str(ROOT / "packages/atlasctl/src")}
    extra: list[str] = []
    if os.environ.get("BIJUX_SCRIPTS_TEST_NO_NETWORK") == "1":
        extra.append("--no-network")
    return subprocess.run(
        [sys.executable, "-m", "atlasctl.cli", *extra, *args],
        cwd=ROOT,
        env=env,
        text=True,
        capture_output=True,
        check=False,
    )


def test_mkdocs_nav_parser_golden() -> None:
    text = (ROOT / "packages/atlasctl/tests/fixtures/docs/mkdocs-nav.good.yml").read_text(encoding="utf-8")
    refs = _mkdocs_nav_file_refs(text)
    assert refs == ["index.md", "operations/index.md", "operations/runbooks.md"]


def test_docs_nav_check_bad_fixture(tmp_path: Path) -> None:
    (tmp_path / "docs").mkdir(parents=True)
    (tmp_path / "docs/index.md").write_text("# Index\n", encoding="utf-8")
    (tmp_path / "mkdocs.yml").write_text(
        "site_name: Test\nnav:\n  - Home: index.md\n  - Missing: missing.md\n",
        encoding="utf-8",
    )

    from atlasctl.commands.docs.legacy import _mkdocs_missing_files

    missing = _mkdocs_missing_files(tmp_path)
    assert missing == ["missing.md"]


def test_docs_check_json_integration() -> None:
    proc = _run_cli("docs", "check", "--report", "json", "--fail-fast")
    payload = json.loads(proc.stdout)
    assert payload["tool"] == "bijux-atlas"
    assert payload["schema_version"] == 1
    assert payload["status"] in {"pass", "fail"}


def test_docs_inventory_and_evidence_policy_generation(tmp_path: Path) -> None:
    inv_out_rel = f"artifacts/scripts/docs-inventory-{tmp_path.name}.md"
    inv = _run_cli("docs", "inventory", "--report", "json", "--out", inv_out_rel)
    assert inv.returncode == 0, inv.stderr
    inv_payload = json.loads(inv.stdout)
    assert inv_payload["output"] == inv_out_rel

    pol_out_rel = f"artifacts/scripts/evidence-policy-{tmp_path.name}.md"
    pol = _run_cli("docs", "evidence-policy-page", "--report", "json", "--out", pol_out_rel)
    assert pol.returncode == 0, pol.stderr
    pol_payload = json.loads(pol.stdout)
    assert pol_payload["output"] == pol_out_rel


def test_docs_generate_json_integration() -> None:
    first = _run_cli("docs", "generate", "--report", "json")
    second = _run_cli("docs", "generate", "--report", "json")
    assert first.returncode in {0, 1}, first.stderr
    assert second.returncode in {0, 1}, second.stderr
    p1 = json.loads(first.stdout)
    p2 = json.loads(second.stdout)
    assert "generated_count" in p1
    assert p1["generated_count"] == p2["generated_count"]


def test_docs_openapi_examples_check_json() -> None:
    proc = _run_cli("docs", "openapi-examples-check", "--report", "json")
    assert proc.returncode in {0, 1}, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["schema_version"] == 1
    assert payload["status"] in {"pass", "fail"}


def test_docs_observability_surface_check_json() -> None:
    proc = _run_cli("docs", "observability-surface-check", "--report", "json")
    assert proc.returncode in {0, 1}, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["schema_version"] == 1
    assert payload["status"] in {"pass", "fail"}


def test_docs_runbooks_contract_check_json() -> None:
    proc = _run_cli("docs", "runbooks-contract-check", "--report", "json")
    assert proc.returncode in {0, 1}, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["schema_version"] == 1
    assert payload["status"] in {"pass", "fail"}


def test_docs_ops_policy_checks_json() -> None:
    commands = [
        "ops-readmes-make-only-check",
        "ops-readme-canonical-links-check",
        "ops-doc-duplication-check",
        "docs-make-only-ops-check",
    ]
    for cmd in commands:
        proc = _run_cli("docs", cmd, "--report", "json")
        assert proc.returncode in {0, 1, 10, 20}, proc.stderr
        if not proc.stdout.strip():
            continue
        payload = json.loads(proc.stdout)
        assert payload["schema_version"] == 1
        assert payload["status"] in {"pass", "fail"}


def test_docs_additional_generators_and_contract_check_json() -> None:
    commands = [
        "generate-architecture-map",
        "generate-upgrade-guide",
        "crate-docs-contract-check",
    ]
    for cmd in commands:
        proc = _run_cli("docs", cmd, "--report", "json")
        assert proc.returncode in {0, 1, 10, 20}, proc.stderr
        if not proc.stdout.strip():
            continue
        payload = json.loads(proc.stdout)
        assert payload["schema_version"] == 1
        assert payload["status"] in {"pass", "fail"}
