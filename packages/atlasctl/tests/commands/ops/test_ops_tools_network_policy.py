from __future__ import annotations

import os
from pathlib import Path

from atlasctl.core.context import RunContext
from atlasctl.commands.ops.tools import run_tool

ROOT = Path(__file__).resolve().parents[5]


def _ctx(tmp_path: Path) -> RunContext:
    return RunContext(
        run_id='netpol',
        profile='local',
        repo_root=ROOT,
        evidence_root=tmp_path / 'evidence',
        scripts_artifact_root=tmp_path / 'scripts',
        run_dir=tmp_path / 'run' / 'netpol',
        output_format='text',
        network_mode='allow',
        verbose=False,
        quiet=True,
        require_clean_git=False,
        no_network=False,
        log_json=False,
        git_sha='testsha',
        git_dirty=False,
    )


def test_network_policy_blocks_disallowed_network_invocation(tmp_path: Path) -> None:
    ctx = _ctx(tmp_path)
    old = os.environ.get('ATLASCTL_OPS_NETWORK_FORBID')
    os.environ['ATLASCTL_OPS_NETWORK_FORBID'] = '1'
    try:
        res = run_tool(ctx, ['curl', '-fsSL', 'https://example.com'])
        assert res.code == 2
        assert 'network policy forbids' in res.stderr
    finally:
        if old is None:
            os.environ.pop('ATLASCTL_OPS_NETWORK_FORBID', None)
        else:
            os.environ['ATLASCTL_OPS_NETWORK_FORBID'] = old


def test_network_policy_allows_declared_local_tool_invocation(tmp_path: Path) -> None:
    ctx = _ctx(tmp_path)
    old = os.environ.get('ATLASCTL_OPS_NETWORK_FORBID')
    os.environ['ATLASCTL_OPS_NETWORK_FORBID'] = '1'
    try:
        res = run_tool(ctx, ['helm', 'template', 'atlas', 'ops/chart'])
        assert res.code != 2
    finally:
        if old is None:
            os.environ.pop('ATLASCTL_OPS_NETWORK_FORBID', None)
        else:
            os.environ['ATLASCTL_OPS_NETWORK_FORBID'] = old
