#!/usr/bin/env sh
set -eu

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
OUT_ROOT="${ATLAS_E2E_OUTPUT_ROOT:-$ROOT/artifacts/e2e-datasets}"
STORE_ROOT="${ATLAS_E2E_STORE_ROOT:-$ROOT/artifacts/e2e-store}"

GFF3=""
FASTA=""
FAI=""
RELEASE=""
SPECIES=""
ASSEMBLY=""

while [ "$#" -gt 0 ]; do
  case "$1" in
    --gff3) GFF3="$2"; shift 2 ;;
    --fasta) FASTA="$2"; shift 2 ;;
    --fai) FAI="$2"; shift 2 ;;
    --release) RELEASE="$2"; shift 2 ;;
    --species) SPECIES="$2"; shift 2 ;;
    --assembly) ASSEMBLY="$2"; shift 2 ;;
    *) echo "unknown arg: $1" >&2; exit 2 ;;
  esac
done

if [ -z "$GFF3" ] || [ -z "$FASTA" ] || [ -z "$FAI" ] || [ -z "$RELEASE" ] || [ -z "$SPECIES" ] || [ -z "$ASSEMBLY" ]; then
  echo "required: --gff3 --fasta --fai --release --species --assembly" >&2
  exit 2
fi

mkdir -p "$OUT_ROOT" "$STORE_ROOT"

cargo run -q -p bijux-atlas-cli --bin bijux-atlas -- atlas ingest \
  --gff3 "$GFF3" \
  --fasta "$FASTA" \
  --fai "$FAI" \
  --output-root "$OUT_ROOT" \
  --release "$RELEASE" \
  --species "$SPECIES" \
  --assembly "$ASSEMBLY" \
  --strictness strict \
  --max-threads 1

if ! publish_out="$(
  cargo run -q -p bijux-atlas-cli --bin bijux-atlas -- atlas dataset publish \
    --source-root "$OUT_ROOT" \
    --store-root "$STORE_ROOT" \
    --release "$RELEASE" \
    --species "$SPECIES" \
    --assembly "$ASSEMBLY" 2>&1
)"; then
  case "$publish_out" in
    *"already published"*) : ;;
    *)
      echo "$publish_out" >&2
      exit 1
      ;;
  esac
else
  echo "$publish_out"
fi

CATALOG_PATH="${ATLAS_E2E_CATALOG_PATH:-$OUT_ROOT/catalog.json}"
DATASET_PREFIX="release=$RELEASE/species=$SPECIES/assembly=$ASSEMBLY"
cat >"$CATALOG_PATH" <<EOF
{"datasets":[{"dataset":{"release":"$RELEASE","species":"$SPECIES","assembly":"$ASSEMBLY"},"manifest_path":"$DATASET_PREFIX/derived/manifest.json","sqlite_path":"$DATASET_PREFIX/derived/gene_summary.sqlite"}]}
EOF

cargo run -q -p bijux-atlas-cli --bin bijux-atlas -- atlas catalog publish \
  --store-root "$STORE_ROOT" \
  --catalog "$CATALOG_PATH"

echo "dataset published: $RELEASE/$SPECIES/$ASSEMBLY"
