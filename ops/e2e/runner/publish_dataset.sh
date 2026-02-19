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

STORE_MANIFEST="$STORE_ROOT/release=$RELEASE/species=$SPECIES/assembly=$ASSEMBLY/derived/manifest.json"
if [ -f "$STORE_MANIFEST" ]; then
  python3 - "$STORE_MANIFEST" "$ROOT/configs/policy/policy.json" <<'PY'
import json
import hashlib
import sys
from pathlib import Path

manifest_path = Path(sys.argv[1])
policy_path = Path(sys.argv[2])
data = json.loads(manifest_path.read_text())
checksums = data.get("checksums", {})
sqlite_sha = str(checksums.get("sqlite_sha256", "")).strip()
gff3_sha = str(checksums.get("gff3_sha256", "")).strip()
fasta_sha = str(checksums.get("fasta_sha256", "")).strip()
fai_sha = str(checksums.get("fai_sha256", "")).strip()
manifest_version = str(data.get("manifest_version", "1") or "1")

policy_sha = "unknown"
if policy_path.exists():
    policy_sha = hashlib.sha256(policy_path.read_bytes()).hexdigest()

data["artifact_version"] = str(data.get("artifact_version") or "v1")
data["schema_version"] = str(data.get("schema_version") or manifest_version)
data["manifest_version"] = manifest_version
data["db_schema_version"] = str(data.get("db_schema_version") or data["schema_version"])
data["toolchain_hash"] = str(data.get("toolchain_hash") or "unknown")
data["db_hash"] = str(data.get("db_hash") or sqlite_sha or "unknown")
data["artifact_hash"] = str(data.get("artifact_hash") or data["db_hash"])
data["input_hashes"] = {
    "gff3_sha256": str(data.get("input_hashes", {}).get("gff3_sha256") or gff3_sha or "unknown"),
    "fasta_sha256": str(data.get("input_hashes", {}).get("fasta_sha256") or fasta_sha or "unknown"),
    "fai_sha256": str(data.get("input_hashes", {}).get("fai_sha256") or fai_sha or "unknown"),
    "policy_sha256": str(data.get("input_hashes", {}).get("policy_sha256") or policy_sha),
}

if data["db_hash"] != sqlite_sha and sqlite_sha:
    data["db_hash"] = sqlite_sha
if not data["artifact_hash"]:
    data["artifact_hash"] = data["db_hash"]

manifest_path.write_text(json.dumps(data, separators=(",", ":"), sort_keys=True))
PY
fi

STACK_NS="${ATLAS_STACK_NAMESPACE:-atlas-e2e}"
if command -v kubectl >/dev/null 2>&1; then
  minio_svc="$(kubectl -n "$STACK_NS" get svc minio -o name 2>/dev/null || true)"
  if [ -n "$minio_svc" ]; then
    sync_pod="atlas-store-sync"
    kubectl -n "$STACK_NS" delete pod "$sync_pod" --ignore-not-found >/dev/null 2>&1 || true
    kubectl -n "$STACK_NS" run "$sync_pod" \
      --image=minio/mc:RELEASE.2025-01-17T23-25-50Z \
      --restart=Never \
      --command -- sh -ceu "sleep 300" >/dev/null
    kubectl -n "$STACK_NS" wait --for=condition=Ready pod/"$sync_pod" --timeout=120s >/dev/null
    kubectl -n "$STACK_NS" exec "$sync_pod" -- sh -ceu "
mc alias set local 'http://minio.$STACK_NS.svc.cluster.local:9000' minioadmin minioadmin >/dev/null
mc mb --ignore-existing local/atlas-artifacts >/dev/null
" >/dev/null
    find "$STORE_ROOT" -type f | while read -r file; do
      rel="${file#"$STORE_ROOT"/}"
      compat_rel="$rel"
      case "$rel" in
        release=*"/species="*"/assembly="*)
          compat_rel="${rel#release=}"
          compat_rel="$(printf '%s\n' "$compat_rel" | sed 's#species=##; s#/assembly=#/#')"
          ;;
      esac
      kubectl -n "$STACK_NS" exec -i "$sync_pod" -- sh -ceu "
mc alias set local 'http://minio.$STACK_NS.svc.cluster.local:9000' minioadmin minioadmin >/dev/null
mc pipe local/atlas-artifacts/'$rel' >/dev/null
" <"$file"
      if [ "$compat_rel" != "$rel" ]; then
        kubectl -n "$STACK_NS" exec -i "$sync_pod" -- sh -ceu "
mc alias set local 'http://minio.$STACK_NS.svc.cluster.local:9000' minioadmin minioadmin >/dev/null
mc pipe local/atlas-artifacts/'$compat_rel' >/dev/null
" <"$file"
      fi
    done
    kubectl -n "$STACK_NS" delete pod "$sync_pod" --ignore-not-found >/dev/null 2>&1 || true
  fi
fi

echo "dataset published: $RELEASE/$SPECIES/$ASSEMBLY"
