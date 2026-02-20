"""Compatibility shim for native check implementations.

Canonical implementations are split under `atlasctl.checks.repo.*`.
"""

from ..checks.repo.legacy_native import *  # noqa: F401,F403
