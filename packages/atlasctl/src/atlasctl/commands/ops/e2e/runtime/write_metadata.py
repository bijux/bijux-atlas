from __future__ import annotations
import json, os, subprocess
from atlasctl.core.runtime.repo_root import find_repo_root
from pathlib import Path


def main() -> int:
    root = find_repo_root()
    out = Path(__import__('sys').argv[1] if len(__import__('sys').argv) > 1 else os.environ.get('OPS_RUN_DIR', 'artifacts/ops/run'))
    out.mkdir(parents=True, exist_ok=True)
    payload = {
        'run_id': os.environ.get('RUN_ID', os.environ.get('OPS_RUN_ID', 'local')),
        'git_sha': subprocess.check_output(['git', '-C', str(root), 'rev-parse', 'HEAD'], text=True).strip() if subprocess.call(['git','-C',str(root),'rev-parse','HEAD'], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)==0 else 'unknown',
        'cwd': str(root),
    }
    (out / 'metadata.json').write_text(
        json.dumps(payload, indent=2, sort_keys=True) + "\n",
        encoding='utf-8',
    )
    print(f'wrote {out}/metadata.json')
    return 0

if __name__ == '__main__':
    raise SystemExit(main())
