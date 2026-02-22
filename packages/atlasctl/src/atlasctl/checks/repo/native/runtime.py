from __future__ import annotations

from pathlib import Path

import atlasctl.checks.repo.native.runtime_modules.repo_native_runtime_core as _runtime_core_mod  # noqa: F401
import atlasctl.checks.repo.native.runtime_modules.repo_native_runtime_policies as _runtime_policies_mod  # noqa: F401

_MODULE_DIR = Path(__file__).with_name("runtime_modules")
for _module in ("repo_native_runtime_core.py", "repo_native_runtime_policies.py"):
    exec((_MODULE_DIR / _module).read_text(encoding="utf-8"), globals())
