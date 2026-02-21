from __future__ import annotations

import json

from atlasctl.suite.manifests import load_first_class_suites

from tests.helpers import golden_text, run_atlasctl


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


def test_suite_markers_doc_exists() -> None:
    proc = run_atlasctl("--quiet", "suite", "check", "--json")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    assert not any("suite markers docs drift" in err for err in payload.get("errors", []))


def test_suite_inventory_shape() -> None:
    proc = run_atlasctl("--quiet", "suite", "list", "--json")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    names = {item["name"] for item in payload["first_class_suites"]}
    assert {"docs", "dev", "ops", "policies", "configs", "local", "slow", "refgrade", "ci", "all", "internal", "refgrade_proof"}.issubset(names)


def test_suite_membership_snapshot_golden() -> None:
    suites = load_first_class_suites()
    payload = {
        "schema_version": 1,
        "tool": "atlasctl",
        "kind": "suite-membership-snapshot",
        "suites": [
            {
                "name": name,
                "markers": list(manifest.markers),
                "internal": manifest.internal,
                "check_ids": list(manifest.check_ids),
            }
            for name, manifest in sorted(suites.items())
        ],
    }
    assert json.dumps(payload, sort_keys=True) == golden_text("suite-membership.json.golden")
