from __future__ import annotations

from pathlib import Path
from types import SimpleNamespace

from atlasctl.checks.model import CheckDef
from atlasctl.checks.selectors import apply_selection_criteria, parse_selection_criteria


def _noop(_repo_root: Path) -> tuple[int, list[str]]:
    return 0, []


def test_parse_selection_criteria_reads_common_flags(tmp_path: Path) -> None:
    ns = SimpleNamespace(
        domain_filter="repo",
        id="checks_repo_*",
        select="",
        check_target="",
        marker=["required"],
        tag=["registry"],
        exclude_marker=["internal"],
        exclude_tag=["experimental"],
        owner=["platform"],
        include_internal=False,
        only_slow=False,
        only_fast=True,
        changed_only=False,
        k="registry",
    )
    criteria = parse_selection_criteria(ns, tmp_path)
    assert criteria.domain == "repo"
    assert criteria.id_globs == ("checks_repo_*",)
    assert "required" in criteria.tags
    assert "registry" in criteria.tags
    assert "internal" in criteria.exclude_tags
    assert "experimental" in criteria.exclude_tags
    assert criteria.owners == ("platform",)
    assert criteria.only_fast is True
    assert criteria.query == "registry"


def test_apply_selection_filters_owner_speed_and_tags() -> None:
    checks = [
        CheckDef(
            check_id="checks_repo_registry_integrity",
            domain="repo",
            description="repo registry integrity",
            budget_ms=100,
            fn=_noop,
            tags=("required", "registry"),
            owners=("platform",),
            slow=False,
        ),
        CheckDef(
            check_id="checks_repo_internal_legacy",
            domain="repo",
            description="repo internal legacy",
            budget_ms=100,
            fn=_noop,
            tags=("internal",),
            owners=("platform",),
            slow=False,
        ),
        CheckDef(
            check_id="checks_ops_runtime_drift",
            domain="ops",
            description="ops runtime drift",
            budget_ms=100,
            fn=_noop,
            tags=("required",),
            owners=("ops",),
            slow=True,
        ),
    ]
    ns = SimpleNamespace(
        domain_filter="repo",
        id="checks_repo_*",
        select="",
        check_target="",
        marker=["required"],
        tag=[],
        exclude_marker=[],
        exclude_tag=[],
        owner=["platform"],
        include_internal=False,
        only_slow=False,
        only_fast=True,
        changed_only=False,
        k="",
    )
    criteria = parse_selection_criteria(ns, Path(".").resolve())
    selected = apply_selection_criteria(checks, criteria)
    assert [item.check_id for item in selected] == ["checks_repo_registry_integrity"]


def test_parse_selection_criteria_ignores_unset_check_target() -> None:
    ns = SimpleNamespace(
        domain_filter="",
        id="",
        select="",
        check_target=None,
        marker=[],
        tag=[],
        exclude_marker=[],
        exclude_tag=[],
        owner=[],
        include_internal=True,
        only_slow=False,
        only_fast=False,
        changed_only=False,
        k="",
    )
    criteria = parse_selection_criteria(ns, Path(".").resolve())
    assert criteria.id_globs == ()


def test_apply_selection_matches_dotted_and_canonical_id_forms() -> None:
    checks = [
        CheckDef(
            check_id="repo.root_shape",
            domain="repo",
            description="root shape",
            budget_ms=100,
            fn=_noop,
            tags=("required",),
            owners=("platform",),
        )
    ]
    ns = SimpleNamespace(
        domain_filter="repo",
        id="checks_repo_root_shape",
        select="",
        check_target="",
        marker=[],
        tag=[],
        exclude_marker=[],
        exclude_tag=[],
        owner=[],
        include_internal=True,
        only_slow=False,
        only_fast=False,
        changed_only=False,
        k="",
    )
    criteria = parse_selection_criteria(ns, Path(".").resolve())
    selected = apply_selection_criteria(checks, criteria)
    assert [item.check_id for item in selected] == ["repo.root_shape"]
