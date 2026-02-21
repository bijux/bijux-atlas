from __future__ import annotations

from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / "packages/atlasctl/src"))

from atlasctl.core.exec_shell import run_shell_script


def test_run_shell_script_captures_stdout_stderr_and_status(tmp_path: Path) -> None:
    script = tmp_path / "echo.sh"
    script.write_text("#!/usr/bin/env bash\nset -euo pipefail\necho out\necho err >&2\n", encoding="utf-8")
    script.chmod(0o755)
    payload = run_shell_script(script, cwd=tmp_path)
    assert payload["status"] == "ok"
    assert payload["exit_code"] == 0
    assert payload["stdout"] == "out\n"
    assert payload["stderr"] == "err\n"


def test_run_shell_script_handles_paths_with_spaces(tmp_path: Path) -> None:
    space_dir = tmp_path / "dir with spaces"
    space_dir.mkdir(parents=True, exist_ok=True)
    script = space_dir / "say.sh"
    script.write_text("#!/usr/bin/env bash\nset -euo pipefail\necho \"$1\"\n", encoding="utf-8")
    script.chmod(0o755)
    payload = run_shell_script(script, args=["ok"], cwd=tmp_path)
    assert payload["status"] == "ok"
    assert payload["stdout"] == "ok\n"
