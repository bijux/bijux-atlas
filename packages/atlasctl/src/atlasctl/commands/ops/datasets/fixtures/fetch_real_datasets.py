from __future__ import annotations
import hashlib, json, pathlib, shutil, subprocess, sys, tarfile

from atlasctl.core.runtime.repo_root import find_repo_root


def main() -> int:
    root = find_repo_root()
    manifest_path = root / 'datasets/real-datasets.json'
    out_root = pathlib.Path(__import__('os').environ.get('ATLAS_REALDATA_ROOT', str(root / 'artifacts/real-datasets')))
    tmp = out_root / '_downloads'
    out_root.mkdir(parents=True, exist_ok=True); tmp.mkdir(parents=True, exist_ok=True)
    manifest = json.loads(manifest_path.read_text(encoding='utf-8'))
    by_id = {d['id']: d for d in manifest['datasets']}
    def dataset_dir(dataset_id: str) -> pathlib.Path:
        release, species, assembly = dataset_id.split('/')
        return out_root / release / species / assembly
    for ds in manifest['datasets']:
        did = ds['id']; ddir = dataset_dir(did); ddir.mkdir(parents=True, exist_ok=True)
        if ds['kind'] == 'download':
            archive = tmp / ds['archive']
            if not archive.exists():
                if pathlib.Path(ds['url']).exists(): shutil.copyfile(ds['url'], archive)
                else: subprocess.check_call(['curl','-fsSL', ds['url'], '-o', str(archive)])
            sha = hashlib.sha256(archive.read_bytes()).hexdigest()
            if sha != ds['sha256']:
                raise SystemExit(f"checksum mismatch for {did}: {sha} != {ds['sha256']}")
            with tarfile.open(archive, 'r:gz') as t: t.extractall(ddir)
        elif ds['kind'] == 'derived':
            src = dataset_dir(ds['derived_from'])
            if not src.exists(): raise SystemExit(f'missing source dataset for derived entry {did}: {src}')
            shutil.copytree(src, ddir, dirs_exist_ok=True)
            transform = root / ds['transform']
            if transform.suffix == '.py':
                subprocess.check_call(['python3', str(transform), str(ddir)])
            else:
                subprocess.check_call([str(transform), str(ddir)])
        else:
            raise SystemExit(f"unknown dataset kind: {ds['kind']}")
    print(f'real datasets ready in {out_root}')
    return 0

if __name__ == '__main__': raise SystemExit(main())
