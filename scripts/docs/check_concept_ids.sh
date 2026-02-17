#!/usr/bin/env sh
set -eu

for f in \
  docs/architecture/boundaries.md \
  docs/architecture/effects.md \
  docs/product/immutability-and-aliases.md \
  docs/reference/querying/pagination.md \
  docs/reference/store/integrity-model.md \
  docs/reference/registry/federation-semantics.md
 do
  [ -f "$f" ] || { echo "missing concept page: $f" >&2; exit 1; }
  grep -q '^Concept ID:' "$f" || { echo "$f missing Concept ID" >&2; exit 1; }
done

echo "concept id check passed"
