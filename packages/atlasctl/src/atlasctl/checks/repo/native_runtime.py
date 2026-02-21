from __future__ import annotations

from pathlib import Path

_MODULE_DIR = Path(__file__).with_name("native_runtime_modules")
for _module in ("repo_native_runtime_core.py", "repo_native_runtime_policies.py"):
    exec((_MODULE_DIR / _module).read_text(encoding="utf-8"), globals())
