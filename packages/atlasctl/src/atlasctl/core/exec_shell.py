"""Compatibility shim for `atlasctl.core.exec_shell`.

LEGACY shim (cutoff: 2026-04-01). Prefer the centralized subprocess/shell
adapter path used by runtime/capability enforcement.
"""

from .effects.exec_shell import run_shell_script

__all__ = ["run_shell_script"]
