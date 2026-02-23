from __future__ import annotations

from atlasctl.runtime.context import RunContext as RuntimeRunContext
from atlasctl.core.context import RunContext as CoreRunContext
from atlasctl.runtime import clock as runtime_clock
from atlasctl.runtime import logging as runtime_logging
from atlasctl.runtime import paths as runtime_paths


def test_runtime_context_facade_matches_core() -> None:
    assert RuntimeRunContext is CoreRunContext
    assert hasattr(runtime_paths, 'find_repo_root')
    assert callable(runtime_clock.utc_now_iso)
    assert callable(runtime_logging.log_event)
