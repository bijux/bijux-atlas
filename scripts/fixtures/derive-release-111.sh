#!/usr/bin/env sh
set -eu

DIR="${1:?target dataset dir required}"
GFF3="$DIR/genes.gff3"
[ -f "$GFF3" ] || { echo "missing $GFF3" >&2; exit 1; }

TMP="$DIR/genes.gff3.tmp"
awk '
  BEGIN { removed=0 }
  {
    # remove gB1 + tB1 to produce removed genes/transcripts in diff
    if ($0 ~ /ID=gB1/ || $0 ~ /ID=tB1/) { removed=1; next }
    gsub("gene_biotype=lncRNA", "gene_biotype=protein_coding", $0)
    print $0
  }
  END {
    print "chrA\tsource\tgene\t45\t55\t.\t+\t.\tID=gA3;Name=GENEA3;biotype=protein_coding"
    print "chrA\tsource\tmRNA\t45\t55\t.\t+\t.\tID=tA3;Parent=gA3"
  }
' "$GFF3" > "$TMP"
mv "$TMP" "$GFF3"

echo "derived release 111 dataset in $DIR"
