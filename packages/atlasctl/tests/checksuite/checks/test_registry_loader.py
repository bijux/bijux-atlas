from __future__ import annotations

import json
from pathlib import Path

import pytest

from atlasctl.checks.registry import Registry, detect_registry_drift, load_registry_generated_json, load_registry_toml


def _write(path: Path, payload: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(payload, encoding="utf-8")


def test_registry_generated_deterministic_ordering(tmp_path: Path) -> None:
    _write(
        tmp_path / "packages/atlasctl/src/atlasctl/checks/REGISTRY.generated.json",
        json.dumps(
            {
                "schema_version": 1,
                "kind": "atlasctl-checks-registry",
                "checks": [
                    {"id": "checks_ops_b", "domain": "ops", "description": "b", "groups": ["ops"]},
                    {"id": "checks_ops_a", "domain": "ops", "description": "a", "groups": ["ops"]},
                ],
            }
        ),
    )
    _write(tmp_path / "configs/policy/check-id-migration.json", json.dumps({"check_ids_alias_expires_on": "2099-01-01", "check_ids": {}}))
    reg = load_registry_generated_json(tmp_path / "packages/atlasctl/src/atlasctl/checks/REGISTRY.generated.json")
    assert [row.id for row in reg.list_checks()] == ["checks_ops_a", "checks_ops_b"]


def test_registry_duplicate_id_detection() -> None:
    reg = Registry(
        records=(
            type("Rec", (), {"id": "checks_ops_a", "domain": "ops", "title": "a", "tags": ("ops",), "speed": "fast", "visibility": "public"})(),
            type("Rec", (), {"id": "checks_ops_a", "domain": "ops", "title": "a", "tags": ("ops",), "speed": "fast", "visibility": "public"})(),
        )
    )
    errors = reg.validate(today=None)
    assert any("duplicate check id" in row for row in errors)


def test_registry_unknown_domain_detection(tmp_path: Path) -> None:
    _write(
        tmp_path / "packages/atlasctl/src/atlasctl/checks/REGISTRY.generated.json",
        json.dumps(
            {
                "schema_version": 1,
                "kind": "atlasctl-checks-registry",
                "checks": [{"id": "checks_bad_x", "domain": "bad", "description": "x", "groups": ["bad"]}],
            }
        ),
    )
    _write(tmp_path / "configs/policy/check-id-migration.json", json.dumps({"check_ids_alias_expires_on": "2099-01-01", "check_ids": {}}))
    with pytest.raises(Exception):
        load_registry_generated_json(tmp_path / "packages/atlasctl/src/atlasctl/checks/REGISTRY.generated.json")


def test_suite_expansion_determinism() -> None:
    rec = type("Rec", (), {"id": "checks_ops_a", "domain": "ops", "title": "a", "tags": ("ops", "required"), "speed": "fast", "visibility": "public"})
    reg = Registry(
        records=(rec(),),
        suites={"ops_fast": {"markers": ["ops"], "include_checks": [], "exclude_markers": []}},
    )
    assert reg.expand_suite("ops_fast") == ("checks_ops_a",)


def test_compat_mapping_and_expiry_enforcement() -> None:
    rec = type("Rec", (), {"id": "checks_ops_a", "domain": "ops", "title": "a", "tags": ("ops",), "speed": "fast", "visibility": "public"})
    from datetime import date
    from atlasctl.checks.registry import CompatEntry

    reg = Registry(
        records=(rec(),),
        compat=(CompatEntry(old_id="ops.a", new_id="checks_ops_a", expires_on=date(2000, 1, 1)),),
    )
    errs = reg.validate(today=date(2026, 1, 1))
    assert any("compat mapping expired" in row for row in errs)


def test_registry_toml_json_drift_detection(tmp_path: Path) -> None:
    _write(
        tmp_path / "packages/atlasctl/src/atlasctl/checks/REGISTRY.toml",
        "[[checks]]\nid=\"checks_ops_a\"\ndomain=\"ops\"\ndescription=\"a\"\ngroups=[\"ops\"]\n",
    )
    _write(
        tmp_path / "packages/atlasctl/src/atlasctl/checks/REGISTRY.generated.json",
        json.dumps({"schema_version": 1, "kind": "atlasctl-checks-registry", "checks": [{"id": "checks_ops_b", "domain": "ops", "description": "b", "groups": ["ops"]}]})
    )
    _write(tmp_path / "configs/policy/check-id-migration.json", json.dumps({"check_ids_alias_expires_on": "2099-01-01", "check_ids": {}}))
    assert detect_registry_drift(repo_root=tmp_path)


def test_registry_toml_loader_sorts(tmp_path: Path) -> None:
    _write(
        tmp_path / "packages/atlasctl/src/atlasctl/checks/REGISTRY.toml",
        "[[checks]]\nid=\"checks_ops_b\"\ndomain=\"ops\"\ndescription=\"b\"\ngroups=[\"ops\"]\n\n[[checks]]\nid=\"checks_ops_a\"\ndomain=\"ops\"\ndescription=\"a\"\ngroups=[\"ops\"]\n",
    )
    _write(tmp_path / "configs/policy/check-id-migration.json", json.dumps({"check_ids_alias_expires_on": "2099-01-01", "check_ids": {}}))
    reg = load_registry_toml(tmp_path / "packages/atlasctl/src/atlasctl/checks/REGISTRY.toml")
    assert [row.id for row in reg.list_checks()] == ["checks_ops_a", "checks_ops_b"]
