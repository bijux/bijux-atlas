#!/usr/bin/env sh
# Purpose: assert repository root shape and forbid unapproved top-level dirs.
# Inputs: repository root directories.
# Outputs: non-zero exit on drift.
set -eu

ALLOWED='\n.cargo\n.github\n.idea\nartifacts\nbin\ncharts\nconfigs\ncrates\ndatasets\ndocs\ne2e\nfixtures\nload\nmakefiles\nobservability\nops\nscripts\ntarget\nxtask\n'

errors=0
for d in */; do
  name=${d%/}
  [ -d "$name" ] || continue
  case "$ALLOWED" in
    *"\n$name\n"*) ;;
    *)
      echo "root shape check failed: unexpected top-level dir '$name'" >&2
      errors=1
      ;;
  esac
done

[ "$errors" -eq 0 ] || exit 1
echo "root shape check passed"
