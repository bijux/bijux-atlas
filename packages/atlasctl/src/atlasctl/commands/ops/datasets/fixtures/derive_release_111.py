from __future__ import annotations
import sys
from pathlib import Path


def main() -> int:
    if len(sys.argv) < 2:
        print('target dataset dir required', file=sys.stderr); return 1
    d = Path(sys.argv[1])
    gff3 = d / 'genes.gff3'
    if not gff3.is_file():
        print(f'missing {gff3}', file=sys.stderr); return 1
    out_lines = []
    removed = False
    for line in gff3.read_text(encoding='utf-8', errors='replace').splitlines():
        if 'ID=gB1' in line or 'ID=tB1' in line:
            removed = True
            continue
        out_lines.append(line.replace('gene_biotype=lncRNA', 'gene_biotype=protein_coding'))
    out_lines.append('chrA	source	gene	45	55	.	+	.	ID=gA3;Name=GENEA3;biotype=protein_coding')
    out_lines.append('chrA	source	mRNA	45	55	.	+	.	ID=tA3;Parent=gA3')
    gff3.write_text('\n'.join(out_lines) + '\n', encoding='utf-8')
    print(f'derived release 111 dataset in {d}')
    return 0

if __name__ == '__main__': raise SystemExit(main())
