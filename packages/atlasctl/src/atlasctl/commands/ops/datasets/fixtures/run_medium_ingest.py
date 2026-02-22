from __future__ import annotations
import subprocess, sys
from pathlib import Path


def main() -> int:
    root = Path('ops/fixtures/medium/v1/data')
    if not root.is_dir():
        print('missing medium fixture data; run make fetch-fixtures', file=sys.stderr)
        return 1
    sharded = len(sys.argv) > 1 and sys.argv[1] == '--sharded'
    cmd = ['cargo','run','-p','bijux-atlas-cli','--bin','bijux-atlas','--','atlas','ingest','--gff3',str(root/'genes.gff3'),'--fasta',str(root/'genome.fa'),'--fai',str(root/'genome.fa.fai'),'--output-root','artifacts/medium-output','--release','110','--species','homo_sapiens','--assembly','GRCh38','--strictness','lenient','--duplicate-gene-id-policy','dedupe']
    if sharded:
        cmd += ['--sharding-plan','contig','--emit-shards']
    return subprocess.call(cmd)

if __name__ == '__main__': raise SystemExit(main())
