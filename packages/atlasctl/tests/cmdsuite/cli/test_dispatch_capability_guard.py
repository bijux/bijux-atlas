from __future__ import annotations

import argparse
import pytest

from atlasctl.cli.dispatch import dispatch_command
from atlasctl.core.errors import ScriptError


class _Ctx:
    run_id = 't'
    repo_root = None
    evidence_root = None
    output_format = 'text'
    git_sha = 'abc'
    profile = 'local'
    network_mode = 'forbid'


def test_dispatch_rejects_unknown_command_before_execution() -> None:
    ns = argparse.Namespace(cmd='unknown-cmd')
    with pytest.raises(ScriptError):
        dispatch_command(_Ctx(), ns, False, lambda *a, **k: None, lambda **k: {}, lambda *a, **k: None, {}, lambda: 'atlasctl 0')
