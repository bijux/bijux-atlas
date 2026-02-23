from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]


def _run(*args: str) -> subprocess.CompletedProcess[str]:
    env = {"PYTHONPATH": str(ROOT / "packages/atlasctl/src")}
    extra: list[str] = []
    if os.environ.get("BIJUX_SCRIPTS_TEST_NO_NETWORK") == "1":
        extra.append("--no-network")
    return subprocess.run([sys.executable, "-m", "atlasctl.cli", *extra, *args], cwd=ROOT, env=env, text=True, capture_output=True, check=False)


def test_check_list_json_inventory() -> None:
    proc = _run("--json", "check", "list")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["status"] == "ok"
    assert any(c["id"] == "checks_repo_no_xtask_refs" for c in payload["checks"])
    assert any(c["id"] == "checks_repo_no_direct_python_invocations" for c in payload["checks"])
    assert any(c["id"] == "checks_repo_public_api_exports" for c in payload["checks"])
    assert any(c["id"] == "checks_license_file_mit" for c in payload["checks"])
    assert any(c["id"] == "checks_checks_registry_integrity" for c in payload["checks"])


def test_check_explain_json() -> None:
    proc = _run("--json", "check", "explain", "checks_repo_no_xtask_refs")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["id"] == "checks_repo_no_xtask_refs"
    assert "how_to_fix" in payload


def test_checks_list_taxonomy_json() -> None:
    proc = _run("--json", "checks", "list")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["schema_name"] == "atlasctl.check-taxonomy.v1"
    assert any(item["domain"] == "repo" for item in payload["taxonomy"])


def test_checks_tree_and_owners_json() -> None:
    tree = _run("--json", "checks", "tree")
    assert tree.returncode == 0, tree.stderr
    tree_payload = json.loads(tree.stdout)
    assert tree_payload["status"] == "ok"
    assert "tree" in tree_payload

    owners = _run("--json", "checks", "owners")
    assert owners.returncode == 0, owners.stderr
    owner_payload = json.loads(owners.stdout)
    assert owner_payload["kind"] == "check-owners"
    assert any(row["owner"] == "platform" for row in owner_payload["owners"])


def test_owners_list_and_validate_json() -> None:
    owners = _run("--json", "owners", "list")
    assert owners.returncode == 0, owners.stderr
    owners_payload = json.loads(owners.stdout)
    assert owners_payload["kind"] == "owners-list"
    assert any(row["id"] == "platform" for row in owners_payload["owners"])

    validate = _run("--json", "owners", "validate")
    assert validate.returncode == 0, validate.stderr
    validate_payload = json.loads(validate.stdout)
    assert validate_payload["kind"] == "owners-validate"
    assert validate_payload["status"] == "pass"


def test_checks_groups_and_slow_json() -> None:
    groups = _run("--json", "checks", "groups")
    assert groups.returncode == 0, groups.stderr
    groups_payload = json.loads(groups.stdout)
    assert groups_payload["kind"] == "check-groups"
    assert any(row["group"] == "required" for row in groups_payload["groups"])

    slow = _run("--json", "checks", "slow")
    assert slow.returncode == 0, slow.stderr
    slow_payload = json.loads(slow.stdout)
    assert slow_payload["kind"] == "check-slow"


def test_check_repo_module_size_alias() -> None:
    proc = _run("check", "repo", "module-size")
    assert proc.returncode in (0, 1), proc.stderr


def test_check_license_alias() -> None:
    proc = _run("--json", "check", "license")
    assert proc.returncode in (0, 1), proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["domain"] == "license"
    assert payload["status"] in {"pass", "fail"}


def test_check_run_selector_flags() -> None:
    by_id = _run("check", "run", "--id", "checks_checks_registry_integrity", "--quiet")
    assert by_id.returncode in (0, 1), by_id.stderr
    assert "checks_checks_registry_integrity" in by_id.stdout

    by_k = _run("check", "run", "-k", "registry_integrity", "--quiet")
    assert by_k.returncode in (0, 1), by_k.stderr
    assert "checks_checks_registry_integrity" in by_k.stdout

    by_domain = _run("check", "run", "--domain", "checks", "--quiet")
    assert by_domain.returncode in (0, 1), by_domain.stderr
    assert "checks_checks_registry_integrity" not in by_domain.stdout
    by_domain_all = _run("check", "run", "--domain", "checks", "--all", "--quiet")
    assert by_domain_all.returncode in (0, 1), by_domain_all.stderr
    assert "checks_checks_registry_integrity" in by_domain_all.stdout


def test_check_run_category_and_domain_filters() -> None:
    proc = _run("check", "run", "--category", "lint", "--domain", "ops", "--quiet")
    assert proc.returncode in (0, 1), proc.stderr
    assert "checks_ops_" in proc.stdout


def test_lint_alias_delegates_to_check_selector() -> None:
    proc = _run("lint", "ops", "--fail-fast")
    assert proc.returncode in (0, 1), proc.stderr
    assert "checks_ops_" in proc.stdout or "\"checks_ops_" in proc.stdout


def test_migrate_checks_registry_json() -> None:
    proc = _run("--json", "migrate", "checks-registry")
    assert proc.returncode in (0, 1), proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["kind"] == "migrate-checks-registry"
