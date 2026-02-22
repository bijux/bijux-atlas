from __future__ import annotations
import hashlib, shutil, subprocess, sys, tarfile
from pathlib import Path


def main() -> int:
    lock = Path('ops/fixtures/medium/v1/manifest.lock')
    if not lock.is_file():
        print(f'missing {lock}', file=sys.stderr); return 1
    kv = {}
    for line in lock.read_text(encoding='utf-8').splitlines():
        if '=' in line:
            k,v = line.split('=',1); kv[k]=v
    url = kv.get('url',''); sha = kv.get('sha256',''); archive = kv.get('archive',''); extract_dir = kv.get('extract_dir','')
    tmp = Path('artifacts/fixtures'); tmp.mkdir(parents=True, exist_ok=True)
    out = tmp / archive
    if Path(url).is_file():
        shutil.copyfile(url, out)
    else:
        subprocess.check_call(['curl','-fsSL',url,'-o',str(out)])
    actual = hashlib.sha256(out.read_bytes()).hexdigest()
    if actual != sha:
        print(f'checksum mismatch: {actual} != {sha}', file=sys.stderr); return 1
    Path(extract_dir).mkdir(parents=True, exist_ok=True)
    with tarfile.open(out, 'r:gz') as t:
        t.extractall(extract_dir)
    print(f'fetched medium fixture to {extract_dir}')
    return 0

if __name__ == '__main__': raise SystemExit(main())
