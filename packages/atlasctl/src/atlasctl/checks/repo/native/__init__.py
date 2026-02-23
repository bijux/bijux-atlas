from __future__ import annotations

from .modules import repo_checks_make_and_layout as _make_layout
from .modules import repo_checks_ops_workflows as _ops_workflows
from .modules import repo_checks_scripts_and_docker as _scripts_docker


def _export_from(module: object) -> dict[str, object]:
    out: dict[str, object] = {}
    for name in dir(module):
        if name.startswith("check_") or name.startswith("generate_") or name.startswith("_find_"):
            out[name] = getattr(module, name)
    return out


globals().update(_export_from(_make_layout))
globals().update(_export_from(_ops_workflows))
globals().update(_export_from(_scripts_docker))

__all__ = sorted(name for name in globals() if name.startswith("check_") or name.startswith("generate_") or name.startswith("_find_"))
