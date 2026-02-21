"""Compatibility shim for `atlasctl.core.exec_shell`."""

from .effects.exec_shell import run_shell_script

__all__ = ["run_shell_script"]
