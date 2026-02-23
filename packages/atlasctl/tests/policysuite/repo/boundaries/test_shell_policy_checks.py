from __future__ import annotations

# BYPASS_TEST_OK: shell boundary tests intentionally reference policy allowlist fixtures.
from pathlib import Path

from atlasctl.checks.repo.enforcement.shell_policy import (
    check_core_no_bash_subprocess,
    check_shell_location_policy,
)


def test_shell_location_policy_fails_when_sh_under_src(tmp_path: Path) -> None:
    bad = tmp_path / "packages/atlasctl/src/atlasctl/foo/probe.sh"
    bad.parent.mkdir(parents=True, exist_ok=True)
    bad.write_text("#!/usr/bin/env bash\nset -euo pipefail\necho ok\n", encoding="utf-8")
    code, errors = check_shell_location_policy(tmp_path)
    assert code == 1
    assert any("probe.sh" in err for err in errors)


def test_shell_location_policy_passes_when_only_vendored(tmp_path: Path) -> None:
    good = tmp_path / "ops/vendor/layout-checks/probe.sh"
    good.parent.mkdir(parents=True, exist_ok=True)
    good.write_text("#!/usr/bin/env bash\nset -euo pipefail\necho ok\n", encoding="utf-8")
    code, errors = check_shell_location_policy(tmp_path)
    assert code == 0, errors


def test_core_no_bash_subprocess_fails_without_allowlist(tmp_path: Path) -> None:
    core = tmp_path / "packages/atlasctl/src/atlasctl/core/probe.py"
    core.parent.mkdir(parents=True, exist_ok=True)
    core.write_text(
        "import subprocess\nsubprocess.run(['bash', '-lc', 'echo hi'], check=False)\n",
        encoding="utf-8",
    )
    code, errors = check_core_no_bash_subprocess(tmp_path)
    assert code == 1
    assert any("probe.py" in err for err in errors)


def test_core_no_bash_subprocess_respects_allowlist(tmp_path: Path) -> None:
    core = tmp_path / "packages/atlasctl/src/atlasctl/core/probe.py"
    core.parent.mkdir(parents=True, exist_ok=True)
    core.write_text(
        "import subprocess\nsubprocess.run(['bash', '-lc', 'echo hi'], check=False)\n",
        encoding="utf-8",
    )
    allowlist = tmp_path / "configs/policy/shell-probes-allowlist.txt"
    allowlist.parent.mkdir(parents=True, exist_ok=True)
    allowlist.write_text("packages/atlasctl/src/atlasctl/core/probe.py\n", encoding="utf-8")
    code, errors = check_core_no_bash_subprocess(tmp_path)
    assert code == 0, errors
