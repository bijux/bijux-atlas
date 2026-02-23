from __future__ import annotations

from atlasctl.runtime.context import RunContext as RuntimeRunContext
from atlasctl.core.context import RunContext as CoreRunContext
from atlasctl.runtime import paths as runtime_paths


def test_runtime_context_facade_matches_core() -> None:
    assert RuntimeRunContext is CoreRunContext
    assert hasattr(runtime_paths, 'find_repo_root')
