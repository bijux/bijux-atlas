#!/usr/bin/env sh
set -eu

# Query planner purity: no DB/network/process IO in pure planning modules.
for f in \
  crates/bijux-atlas-query/src/planner.rs \
  crates/bijux-atlas-query/src/filters.rs \
  crates/bijux-atlas-query/src/cost.rs \
  crates/bijux-atlas-query/src/limits.rs
 do
  for pat in 'rusqlite' 'reqwest' 'std::fs' 'tokio::net' 'std::process'; do
    if rg -n "$pat" "$f" >/dev/null; then
      echo "effects-lint: forbidden '$pat' in $f" >&2
      exit 1
    fi
  done
done

# HTTP layer should not do raw file IO directly; use http/effects_adapters.rs.
for f in $(find crates/bijux-atlas-server/src/http -type f -name '*.rs' ! -name 'effects_adapters.rs'); do
  if rg -n 'std::fs::|use std::fs::|File::open\(' "$f" >/dev/null; then
    echo "effects-lint: raw fs IO forbidden in $f; use http/effects_adapters.rs" >&2
    exit 1
  fi
done

echo "effects lint passed"
