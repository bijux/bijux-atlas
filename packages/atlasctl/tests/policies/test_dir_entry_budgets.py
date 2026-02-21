from __future__ import annotations

import json
from pathlib import Path

from atlasctl.policies.dir_entry_budgets import evaluate_budget


def _write_exceptions(repo: Path, payload: dict[str, object]) -> None:
    path = repo / "configs/policy/BUDGET_EXCEPTIONS.yml"
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True), encoding="utf-8")


def test_py_file_budget_excludes_init_and_entry_budget_excludes_pycache(tmp_path: Path) -> None:
    pkg = tmp_path / "packages/atlasctl/src/atlasctl/domain"
    pkg.mkdir(parents=True)
    (pkg / "__init__.py").write_text("", encoding="utf-8")
    for i in range(10):
        (pkg / f"m{i}.py").write_text("x = 1\n", encoding="utf-8")
    (pkg / "__pycache__").mkdir()
    _write_exceptions(tmp_path, {"schema_version": 1, "max_exceptions": 3, "exceptions": []})

    py_payload = evaluate_budget(tmp_path, "py-files-per-dir")
    entry_payload = evaluate_budget(tmp_path, "entries-per-dir")
    domain_py = next(item for item in py_payload["items"] if item["path"].endswith("/domain"))
    domain_entry = next(item for item in entry_payload["items"] if item["path"].endswith("/domain"))

    assert domain_py["count"] == 10
    assert domain_py["status"] == "warn"
    assert domain_entry["count"] == 11  # __init__ + ten modules, __pycache__ excluded
    assert domain_entry["status"] == "fail"


def test_exception_registry_requires_owner_and_non_expired(tmp_path: Path) -> None:
    pkg = tmp_path / "packages/atlasctl/src/atlasctl/domain"
    pkg.mkdir(parents=True)
    _write_exceptions(
        tmp_path,
        {
            "schema_version": 1,
            "max_exceptions": 1,
            "exceptions": [
                {"path": "packages/atlasctl/src/atlasctl/domain", "owner": "", "reason": "temporary", "expires_on": "2001-01-01"},
                {"path": "packages/atlasctl/src/atlasctl/other", "owner": "team-x", "reason": "temporary", "expires_on": "2099-01-01"},
            ],
        },
    )

    payload = evaluate_budget(tmp_path, "entries-per-dir")
    first = payload["items"][0]
    assert first["path"] == "_exceptions_"
    errs = first["errors"]
    assert any("missing owner" in err for err in errs)
    assert any("expired on" in err for err in errs)
    assert any("exception count exceeded" in err for err in errs)
