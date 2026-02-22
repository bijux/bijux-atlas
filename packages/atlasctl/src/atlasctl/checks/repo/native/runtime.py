from __future__ import annotations

from pathlib import Path

import atlasctl.checks.repo.native.modules.repo_checks_make_and_layout as _make_layout_mod  # noqa: F401
import atlasctl.checks.repo.native.modules.repo_checks_scripts_and_docker as _scripts_docker_mod  # noqa: F401
from .runtime_modules import repo_native_runtime_core as _runtime_core_mod  # noqa: F401
from .runtime_modules import repo_native_runtime_policies as _runtime_policies_mod  # noqa: F401

_MODULE_DIR = Path(__file__).with_name("runtime_modules")
for _module in ("repo_native_runtime_core.py", "repo_native_runtime_policies.py"):
    exec((_MODULE_DIR / _module).read_text(encoding="utf-8"), globals())
