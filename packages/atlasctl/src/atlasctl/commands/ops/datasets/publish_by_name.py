from __future__ import annotations
import os
import subprocess
import sys
from pathlib import Path


def _publish(root: Path, gff3: Path, fasta: Path, fai: Path, release: str, species: str, assembly: str) -> int:
    return subprocess.call([
        'python3', str(root / 'packages/atlasctl/src/atlasctl/commands/ops/e2e/runtime/publish_dataset.py'),
        '--gff3', str(gff3),
        '--fasta', str(fasta),
        '--fai', str(fai),
        '--release', release,
        '--species', species,
        '--assembly', assembly,
    ])


def main() -> int:
    root = Path.cwd()
    dataset = sys.argv[1] if len(sys.argv) > 1 else 'medium'
    real_root = Path(os.environ.get('ATLAS_REALDATA_ROOT', str(root / 'artifacts/real-datasets')))
    if dataset == 'medium':
        return _publish(
            root,
            root / 'ops/fixtures/medium/v1/data/genes.gff3',
            root / 'ops/fixtures/medium/v1/data/genome.fa',
            root / 'ops/fixtures/medium/v1/data/genome.fa.fai',
            '110', 'homo_sapiens', 'GRCh38',
        )
    if dataset in {'real1', 'real110', 'real111'}:
        subprocess.check_call(['bash', str(root / 'ops/datasets/scripts/fixtures/fetch-real-datasets.sh')], stdout=subprocess.DEVNULL)
        rel = '111' if dataset in {'real1', 'real111'} else '110'
        return _publish(
            root,
            real_root / rel / 'homo_sapiens' / 'GRCh38' / 'genes.gff3',
            real_root / rel / 'homo_sapiens' / 'GRCh38' / 'genome.fa',
            real_root / rel / 'homo_sapiens' / 'GRCh38' / 'genome.fa.fai',
            rel, 'homo_sapiens', 'GRCh38',
        )
    print('unsupported DATASET=%s (expected: medium|real1|real110|real111)' % dataset, file=sys.stderr)
    return 2


if __name__ == '__main__':
    raise SystemExit(main())
