from __future__ import annotations
import argparse
from atlasctl.core.runtime.repo_root import find_repo_root
import json, os, re, subprocess, sys, time
from pathlib import Path

ROOT = find_repo_root()


def _read_manifest_field(name: str, field: str):
    data = json.loads((ROOT/'ops/obs/drills/drills.json').read_text(encoding='utf-8'))
    for d in data.get('drills', []):
        if d.get('name') == name:
            return d.get(field)
    print(f'drill not found: {name}', file=sys.stderr)
    raise SystemExit(3)


def _cleanup():
    subprocess.call(
        ['python3', 'packages/atlasctl/src/atlasctl/commands/ops/stack/faults/inject.py', 'block-minio', 'off'],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    ns = os.environ.get('ATLAS_NS', os.environ.get('ATLAS_E2E_NAMESPACE','atlas-e2e'))
    subprocess.call(['kubectl','-n',ns,'delete','pod','toxiproxy-latency','--ignore-not-found'], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("drill_name", nargs="?")
    ap.add_argument("--drill")
    ap.add_argument("--id", dest="drill_id")
    ap.add_argument("--dry-run", action="store_true")
    ns = ap.parse_args()
    drill = ns.drill or ns.drill_id or ns.drill_name or ''
    if not drill:
        print('usage: run_drill.py --id <drill-name>', file=sys.stderr); return 2
    out_dir = ROOT/'artifacts/observability/drills'
    ops_obs_dir = ROOT/'artifacts/ops/obs'
    out_dir.mkdir(parents=True, exist_ok=True); ops_obs_dir.mkdir(parents=True, exist_ok=True)
    script_rel = _read_manifest_field(drill, 'script') or ''
    timeout_seconds = int(_read_manifest_field(drill, 'timeout_seconds') or 120)
    warmup = bool(_read_manifest_field(drill, 'warmup'))
    cleanup = bool(_read_manifest_field(drill, 'cleanup'))
    expected_signals = _read_manifest_field(drill, 'expected_signals') or []
    if not script_rel:
        print(f'manifest missing script for {drill}', file=sys.stderr); return 4
    script_path = ROOT / script_rel
    if warmup and not ns.dry_run:
        base = os.environ.get('ATLAS_BASE_URL','http://127.0.0.1:18080')
        subprocess.call(['curl','-fsS',f'{base}/healthz'], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        subprocess.call(['curl','-fsS',f'{base}/v1/version'], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    started_at = time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime())
    status = 'pass'
    log_snapshot = ops_obs_dir / f'drill-{drill}.logs.txt'
    if ns.dry_run:
        status = 'pass'
        (ops_obs_dir / 'metrics.prom').write_text('', encoding='utf-8')
        (ops_obs_dir / 'traces.snapshot.log').write_text('', encoding='utf-8')
        log_snapshot.write_text('', encoding='utf-8')
    else:
        try:
            proc = subprocess.run(['python3', str(script_path)] if script_path.suffix == '.py' else ['bash', str(script_path)], timeout=timeout_seconds)
            if proc.returncode != 0:
                status = 'fail'
        except subprocess.TimeoutExpired:
            status = 'fail'
        subprocess.call(['python3','packages/atlasctl/src/atlasctl/commands/ops/observability/snapshot_metrics.py', str(ops_obs_dir)])
        subprocess.call(['python3','packages/atlasctl/src/atlasctl/commands/ops/observability/snapshot_traces.py', str(ops_obs_dir)])
        with log_snapshot.open('wb') as f:
            subprocess.call(['kubectl','-n',os.environ.get('ATLAS_E2E_NAMESPACE','atlas-e2e'),'logs','-l',f"app.kubernetes.io/instance={os.environ.get('ATLAS_E2E_RELEASE_NAME','atlas-e2e')}",'--all-containers','--tail=2000'], stdout=f, stderr=subprocess.DEVNULL)
        if subprocess.call(['python3','packages/atlasctl/src/atlasctl/commands/ops/observability/validate_logs_schema.py','--namespace',os.environ.get('ATLAS_E2E_NAMESPACE','atlas-e2e'),'--release',os.environ.get('ATLAS_E2E_RELEASE_NAME','atlas-e2e'),'--strict-live']) != 0:
            status = 'fail'
    if status == 'pass' and not ns.dry_run:
        metrics = (ops_obs_dir/'metrics.prom').read_text(encoding='utf-8', errors='replace') if (ops_obs_dir/'metrics.prom').exists() else ''
        traces = (ops_obs_dir/'traces.snapshot.log').read_text(encoding='utf-8', errors='replace').lower() if (ops_obs_dir/'traces.snapshot.log').exists() else ''
        logs = '\n'.join(
            p.read_text(encoding='utf-8', errors='replace').lower()
            for p in sorted(ops_obs_dir.glob('drill-*.logs.txt'))
        )
        for signal in expected_signals:
            kind, _, value = signal.partition(':')
            if kind == 'metric' and value not in metrics: status = 'fail'; break
            if kind == 'trace' and value.lower() not in traces: status = 'fail'; break
            if kind == 'log' and value.lower() not in logs: status = 'fail'; break
            if kind in {'validator',''}: continue
    if cleanup and not ns.dry_run:
        _cleanup()
    ended_at = time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime())
    result_file = out_dir / f'{drill}.result.json'
    trace_ids = []
    tp = ops_obs_dir/'traces.snapshot.log'
    if tp.exists():
        ids = sorted(set(re.findall(r'[0-9a-f]{16,32}', tp.read_text(encoding='utf-8', errors='replace'))))
        trace_ids = ids[:20]
    result = {
        'schema_version':1,'drill':drill,'started_at':started_at,'ended_at':ended_at,
        'status':'pass' if status=='pass' else 'fail',
        'snapshot_paths':{'metrics':'artifacts/ops/obs/metrics.prom','traces':'artifacts/ops/obs/traces.snapshot.log','logs':str(log_snapshot)},
        'trace_ids':trace_ids,'expected_signals':expected_signals,
    }
    result_file.write_text(
        json.dumps(result, indent=2, sort_keys=True) + "\n",
        encoding='utf-8',
    )
    schema = json.loads((ROOT/'ops/obs/drills/result.schema.json').read_text(encoding='utf-8'))
    missing = sorted(set(schema.get('required', [])) - set(result))
    if missing:
        print(f'result schema validation failed: missing keys {missing}', file=sys.stderr); return 1
    print('drill result schema validation passed')
    if status != 'pass':
        print(f'drill failed: {drill}', file=sys.stderr); return 1
    print(f'drill passed: {drill} ({result_file})')
    return 0

if __name__ == '__main__': raise SystemExit(main())
