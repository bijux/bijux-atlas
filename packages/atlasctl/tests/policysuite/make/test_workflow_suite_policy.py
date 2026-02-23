from __future__ import annotations

from pathlib import Path

from atlasctl.checks.domains.policies.make.enforcement import check_workflows_no_adhoc_check_run


def test_workflows_no_adhoc_check_run_detects_violation(tmp_path: Path) -> None:
    wf = tmp_path / ".github/workflows/ci.yml"
    wf.parent.mkdir(parents=True, exist_ok=True)
    wf.write_text(
        "jobs:\n"
        "  ci:\n"
        "    steps:\n"
        "      - run: ./bin/atlasctl check run --id checks_repo_example\n",
        encoding="utf-8",
    )
    code, errors = check_workflows_no_adhoc_check_run(tmp_path)
    assert code == 1
    assert any("ad-hoc `atlasctl check run ...`" in err for err in errors)


def test_workflows_no_adhoc_check_run_accepts_suite_and_gate(tmp_path: Path) -> None:
    wf = tmp_path / ".github/workflows/ci.yml"
    wf.parent.mkdir(parents=True, exist_ok=True)
    wf.write_text(
        "jobs:\n"
        "  ci:\n"
        "    steps:\n"
        "      - run: ./bin/atlasctl suite run checks-fast --report json\n"
        "      - run: ./bin/atlasctl gate run --preset root --all --report json\n",
        encoding="utf-8",
    )
    code, errors = check_workflows_no_adhoc_check_run(tmp_path)
    assert code == 0
    assert errors == []

