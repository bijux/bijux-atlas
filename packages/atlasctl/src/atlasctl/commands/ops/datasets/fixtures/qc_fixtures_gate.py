from __future__ import annotations
import os, shutil, subprocess
from pathlib import Path


def _run_case(root: Path, out_root: Path, thresholds: Path, case_name: str, gff3: Path, fasta: Path, fai: Path, release: str, species: str, assembly: str) -> None:
    case_out = out_root / case_name
    case_out.mkdir(parents=True, exist_ok=True)
    subprocess.check_call(['cargo','run','-q','-p','bijux-atlas-cli','--bin','bijux-atlas','--','atlas','ingest','--gff3',str(gff3),'--fasta',str(fasta),'--fai',str(fai),'--output-root',str(case_out),'--release',release,'--species',species,'--assembly',assembly,'--strictness','strict','--max-threads','1'])
    qc = case_out / f'release={release}/species={species}/assembly={assembly}/derived/qc.json'
    subprocess.check_call(['cargo','run','-q','-p','bijux-atlas-cli','--bin','bijux-atlas','--','atlas','ingest-validate','--qc-report',str(qc),'--thresholds',str(thresholds)])


def main() -> int:
    root = Path.cwd()
    out_root = root / 'artifacts/isolate/qc-fixtures'
    thresholds = root / 'configs/ops/dataset-qc-thresholds.v1.json'
    shutil.rmtree(out_root, ignore_errors=True)
    out_root.mkdir(parents=True, exist_ok=True)
    _run_case(root, out_root, thresholds, 'minimal', root/'crates/bijux-atlas-ingest/tests/fixtures/tiny/genes.gff3', root/'crates/bijux-atlas-ingest/tests/fixtures/tiny/genome.fa', root/'crates/bijux-atlas-ingest/tests/fixtures/tiny/genome.fa.fai', '110','homo_sapiens','GRCh38')
    _run_case(root, out_root, thresholds, 'medium', root/'ops/fixtures/medium/v1/data/genes.gff3', root/'ops/fixtures/medium/v1/data/genome.fa', root/'ops/fixtures/medium/v1/data/genome.fa.fai', '110','homo_sapiens','GRCh38')
    print('qc fixtures gate passed')
    return 0

if __name__ == '__main__': raise SystemExit(main())
