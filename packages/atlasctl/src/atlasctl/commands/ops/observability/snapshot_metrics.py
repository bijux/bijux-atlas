from __future__ import annotations
import json, os, subprocess, sys
from pathlib import Path


def main() -> int:
    root = Path.cwd()
    base_url = os.environ.get('ATLAS_BASE_URL', 'http://127.0.0.1:18080')
    ns = os.environ.get('ATLAS_E2E_NAMESPACE', 'atlas-e2e')
    release = os.environ.get('ATLAS_E2E_RELEASE_NAME', 'atlas-e2e')
    local_port = os.environ.get('ATLAS_E2E_LOCAL_PORT', '18080')
    out_dir = Path(sys.argv[1] if len(sys.argv) > 1 else str(root / 'artifacts/ops/obs'))
    out_dir.mkdir(parents=True, exist_ok=True)

    curl_base = ['curl', '--connect-timeout', '2', '--max-time', '5', '-fsS']
    used_pf = False
    pf_pid: str | None = None
    health_rc = subprocess.call([*curl_base, f'{base_url}/healthz'], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    if health_rc != 0:
        if subprocess.call(['kubectl', 'config', 'current-context'], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL) != 0:
            (out_dir / 'metrics.prom').write_text('', encoding='utf-8')
            print('metrics snapshot skipped: kubectl context is not configured')
            print(f'wrote {out_dir / "metrics.prom"}')
            return 0
        pod_line = subprocess.check_output(['bash','-lc', f"kubectl -n '{ns}' get pods -l app.kubernetes.io/instance='{release}' --field-selector=status.phase=Running -o name 2>/dev/null | tail -n1 | cut -d/ -f2"], text=True).strip()
        if not pod_line:
            (out_dir / 'metrics.prom').write_text('', encoding='utf-8')
            print(f"metrics snapshot skipped: no running atlas pod found in namespace '{ns}'")
            print(f'wrote {out_dir / "metrics.prom"}')
            return 0
        pf_cmd = f"kubectl -n '{ns}' port-forward 'pod/{pod_line}' {local_port}:8080 >/tmp/atlas-snapshot-metrics-port-forward.log 2>&1 & echo $!"
        pf_pid = subprocess.check_output(['bash','-lc', pf_cmd], text=True).strip()
        base_url = f'http://127.0.0.1:{local_port}'
        used_pf = True
    try:
        with (out_dir / 'metrics.prom').open('wb') as fh:
            rc = subprocess.call([*curl_base, f'{base_url}/metrics'], stdout=fh)
        if rc != 0:
            (out_dir / 'metrics.prom').write_text('', encoding='utf-8')
        payload = {
            'git_sha': os.environ.get('GIT_SHA') or subprocess.check_output(['git', '-C', str(root), 'rev-parse', 'HEAD'], text=True).strip() if subprocess.call(['git','-C',str(root),'rev-parse','HEAD'], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)==0 else 'unknown',
            'image_digest': os.environ.get('ATLAS_IMAGE_DIGEST', 'unknown'),
            'dataset_hash': os.environ.get('ATLAS_DATASET_HASH', 'unknown'),
            'release': os.environ.get('ATLAS_RELEASE', 'unknown'),
        }
        (out_dir / 'baseline-metadata.json').write_text(
            json.dumps(payload, indent=2) + "\n",
            encoding='utf-8',
        )
        print(f'wrote {out_dir / "metrics.prom"}')
        return 0
    finally:
        if used_pf and pf_pid:
            subprocess.call(['bash','-lc', f'kill {pf_pid} >/dev/null 2>&1 || true'])


if __name__ == '__main__':
    raise SystemExit(main())
