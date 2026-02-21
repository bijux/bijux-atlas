from __future__ import annotations

from pathlib import Path

_MODULE_DIR = Path(__file__).with_name("ops_runtime_modules")
for _module in ("ops_runtime_checks.py", "ops_runtime_commands.py"):
    exec((_MODULE_DIR / _module).read_text(encoding="utf-8"), globals())
