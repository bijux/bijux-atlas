from __future__ import annotations

from pathlib import Path

import pytest

from atlasctl.checks.models import (
    CheckContext,
    CheckId,
    CheckResult,
    CheckSpec,
    DomainId,
    EvidenceRef,
    SCHEMA_VERSION,
    Selector,
    Speed,
    Status,
    SuiteId,
    Timing,
    Violation,
    Visibility,
    stable_evidence,
    stable_violations,
    to_json_dict,
    to_json_text,
    validate_tag,
)


def test_schema_version_constant() -> None:
    assert SCHEMA_VERSION == 1


def test_check_id_validator_rejects_bad_values() -> None:
    with pytest.raises(ValueError):
        CheckId.parse("CHECKS_BAD")

    with pytest.raises(ValueError):
        CheckId.parse("checks")

    parsed = CheckId.parse("checks_ops_surface_contract")
    assert str(parsed) == "checks_ops_surface_contract"


def test_domain_validator_membership() -> None:
    assert str(DomainId.parse("ops")) == "ops"
    with pytest.raises(ValueError):
        DomainId.parse("repo")


def test_suite_id_validator() -> None:
    assert str(SuiteId.parse("checks_fast")) == "checks_fast"
    with pytest.raises(ValueError):
        SuiteId.parse("ChecksFast")


def test_tag_validator_bounds_and_format() -> None:
    assert validate_tag("required") == "required"
    with pytest.raises(ValueError):
        validate_tag("bad tag")
    with pytest.raises(ValueError):
        validate_tag("lint", banned_adjectives={"lint"})


def test_check_spec_defaults_visibility_and_speed() -> None:
    spec = CheckSpec(
        id="checks_ops_surface_contract",
        domain="ops",
        title="validate ops surface contract",
        docs="packages/atlasctl/docs/checks/usage.md",
    )
    assert spec.visibility == Visibility.PUBLIC
    assert spec.speed == Speed.FAST


def test_context_requires_run_id() -> None:
    with pytest.raises(ValueError):
        CheckContext(repo_root=Path("."), artifacts_root=Path("artifacts"), run_id="")


def test_stable_ordering_helpers() -> None:
    violations = (
        Violation(code="B", message="two", path="b.txt", line=9),
        Violation(code="A", message="one", path="a.txt", line=1),
    )
    evidence = (
        EvidenceRef(kind="log", path="z.log"),
        EvidenceRef(kind="json", path="a.json"),
    )
    ordered_violations = stable_violations(violations)
    ordered_evidence = stable_evidence(evidence)
    assert ordered_violations[0].path == "a.txt"
    assert ordered_evidence[0].path == "a.json"


def test_check_result_normalizes_types() -> None:
    result = CheckResult(
        id="checks_ops_surface_contract",
        status="fail",
        violations=(Violation(code="OPS_SURFACE", message="bad"),),
        timing=Timing(duration_ms=12, budget_ms=20),
    )
    assert str(result.id) == "checks_ops_surface_contract"
    assert result.status == Status.FAIL


def test_selector_model() -> None:
    selector = Selector(
        ids=(CheckId.parse("checks_ops_surface_contract"),),
        domains=(DomainId.parse("ops"),),
        suites=(SuiteId.parse("checks_fast"),),
        tags=("required",),
        include_internal=False,
    )
    payload = to_json_dict(selector)
    assert payload["tags"] == ["required"]


def test_model_json_roundtrip_text() -> None:
    spec = CheckSpec(
        id="checks_ops_surface_contract",
        domain="ops",
        title="validate ops surface contract",
        docs="packages/atlasctl/docs/checks/usage.md",
        tags=("required", "ops"),
    )
    body = to_json_text(spec)
    assert '"domain": {' in body
    assert '"value": "ops"' in body
