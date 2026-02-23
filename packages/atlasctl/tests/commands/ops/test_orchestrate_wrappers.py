from __future__ import annotations

import json
from pathlib import Path

from atlasctl.core.context import RunContext
from atlasctl.core.schema.schema_utils import validate_json
from atlasctl.commands.ops.orchestrate._wrappers import OrchestrateSpec, run_wrapped, write_wrapper_artifacts
from atlasctl.commands.ops.orchestrate.manifest import (
    DATASETS_WRAPPER_ACTIONS,
    E2E_WRAPPER_ACTIONS,
    K8S_WRAPPER_ACTIONS,
    LOAD_WRAPPER_ACTIONS,
    OBS_WRAPPER_ACTIONS,
    STACK_WRAPPER_ACTIONS,
)
from atlasctl.commands.ops.scenario.command import run_scenario_from_manifest


ROOT = Path(__file__).resolve().parents[5]


def _ctx(tmp_path: Path, run_id: str = 'test-run') -> RunContext:
    evidence = tmp_path / 'evidence'
    return RunContext(
        run_id=run_id,
        profile='local',
        repo_root=ROOT,
        evidence_root=evidence,
        scripts_artifact_root=tmp_path / 'scripts',
        run_dir=(evidence / run_id),
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


def test_wrapper_action_manifests_are_command_arrays() -> None:
    for mapping in (
        K8S_WRAPPER_ACTIONS,
        STACK_WRAPPER_ACTIONS,
        OBS_WRAPPER_ACTIONS,
        LOAD_WRAPPER_ACTIONS,
        E2E_WRAPPER_ACTIONS,
        DATASETS_WRAPPER_ACTIONS,
    ):
        assert mapping
        for _name, cmd in mapping.items():
            assert isinstance(cmd, list)
            assert cmd and all(isinstance(x, str) and x for x in cmd)


def test_wrapper_report_json_is_schema_valid(tmp_path: Path) -> None:
    ctx = _ctx(tmp_path)
    payload = write_wrapper_artifacts(ctx, 'stack', 'up', ['./bin/atlasctl', 'ops', 'stack', 'up'], 0, 'ok')
    report = ROOT / payload['artifacts']['report']
    assert report.exists()
    data = json.loads(report.read_text(encoding='utf-8'))
    validate_json(data, ROOT / 'configs/contracts/scripts-tool-output.schema.json')


def test_wrapper_failure_report_json_is_schema_valid(tmp_path: Path) -> None:
    ctx = _ctx(tmp_path)
    code = run_wrapped(ctx, OrchestrateSpec('stack', 'up', ['__missing_tool__']), 'json')
    assert code == 1
    report = next((tmp_path / 'evidence').rglob('report.json'))
    data = json.loads(report.read_text(encoding='utf-8'))
    validate_json(data, ROOT / 'configs/contracts/scripts-tool-output.schema.json')
    assert data['details']['failure']['kind'] in {'missing-tool', 'exit-code'}


def test_dry_run_produces_no_evidence_writes(tmp_path: Path) -> None:
    ctx = _ctx(tmp_path)
    code = run_wrapped(ctx, OrchestrateSpec('stack', 'up', ['./bin/atlasctl', 'ops', 'stack', 'up']), 'json', dry_run=True)
    assert code == 0
    assert not (tmp_path / 'evidence').exists()


def test_scenario_no_write_produces_no_evidence_writes(tmp_path: Path) -> None:
    ctx = _ctx(tmp_path, run_id='scenario-no-write')
    rel = Path('artifacts/tmp/ops-scenario-test.json')
    manifest = ROOT / rel
    manifest.parent.mkdir(parents=True, exist_ok=True)
    manifest.write_text(
        json.dumps(
            {
                'scenarios': {
                    'sample': {
                        'command': ['./bin/atlasctl', 'ops', 'stack', '--report', 'text', 'up']
                    }
                }
            },
            indent=2,
        )
        + '\n',
        encoding='utf-8',
    )
    try:
        code = run_scenario_from_manifest(ctx, 'json', rel.as_posix(), 'sample', no_write=True)
        assert code == 0
        assert not (tmp_path / 'evidence').exists()
    finally:
        manifest.unlink(missing_ok=True)
