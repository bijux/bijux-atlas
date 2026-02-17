#!/usr/bin/env sh
set -eu

BASE_URL="${ATLAS_E2E_BASE_URL:-http://127.0.0.1:18080}"

queries='\
/healthz\
/readyz\
/v1/version\
/v1/datasets\
/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&limit=1\
/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&gene_id=GENE1\
/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&name=Gene1\
/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&name_prefix=Gene\
/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&biotype=protein_coding\
/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&region=chr1:1-1000\
/v1/genes/count?release=110&species=homo_sapiens&assembly=GRCh38\
/v1/releases/110/species/homo_sapiens/assemblies/GRCh38\
/v1/transcripts/TX1?release=110&species=homo_sapiens&assembly=GRCh38\
/v1/genes/GENE1/transcripts?release=110&species=homo_sapiens&assembly=GRCh38\
/v1/diff/genes?from_release=109&to_release=110&species=homo_sapiens&assembly=GRCh38&limit=10\
/v1/diff/region?from_release=109&to_release=110&species=homo_sapiens&assembly=GRCh38&region=chr1:1-1000\
/v1/sequence/region?release=110&species=homo_sapiens&assembly=GRCh38&region=chr1:1-20\
/v1/genes/GENE1/sequence?release=110&species=homo_sapiens&assembly=GRCh38\
/metrics\
/debug/datasets\
'

for q in $queries; do
  body="$(curl -fsS "$BASE_URL$q")"
  case "$q" in
    /metrics) echo "$body" | grep -q '^bijux_' ;;
    /healthz|/readyz) echo "$body" | grep -q '"status"' ;;
    *) [ -n "$body" ] ;;
  esac
  echo "ok $q"
done
