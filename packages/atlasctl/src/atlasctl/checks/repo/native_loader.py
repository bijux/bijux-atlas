from __future__ import annotations

from pathlib import Path

_MODULE_DIR = Path(__file__).with_name("native_modules")
for _module in ("repo_checks_make_and_layout.py", "repo_checks_scripts_and_docker.py"):
    exec((_MODULE_DIR / _module).read_text(encoding="utf-8"), globals())
