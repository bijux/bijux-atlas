from __future__ import annotations

from pathlib import Path

from atlasctl.checks.domains.policies.make import check_policies_no_plaintext_allowlist_files
from atlasctl.checks.domains.policies.make.enforcement import check_policies_no_legacy_ops_bypass_ledger


def test_no_plaintext_allowlist_files_detects_txt(tmp_path: Path) -> None:
    p = tmp_path / "configs/policy/example-allowlist.txt"
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text("value\n", encoding="utf-8")
    code, errors = check_policies_no_plaintext_allowlist_files(tmp_path)
    assert code == 1
    assert any("plaintext allowlist files are forbidden" in err for err in errors)


def test_no_legacy_ops_bypass_ledger_detects_file(tmp_path: Path) -> None:
    legacy = tmp_path / "ops/_meta/bypass-ledger.json"
    legacy.parent.mkdir(parents=True, exist_ok=True)
    legacy.write_text("{\"entries\":[]}\n", encoding="utf-8")
    code, errors = check_policies_no_legacy_ops_bypass_ledger(tmp_path)
    assert code == 1
    assert any("legacy bypass ledger forbidden" in err for err in errors)

