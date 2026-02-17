#!/usr/bin/env sh
# owner: platform
# purpose: enforce unit tests avoid live network dependencies.
# stability: public
# called-by: make policy-lint
# Purpose: script interface entrypoint.
# Inputs: command-line args and repository files/env as documented by caller.
# Outputs: exit status and deterministic stdout/stderr or generated artifacts.
set -eu

violations=""
for file in $(find crates -path '*/src/*.rs' -type f); do
  if grep -q '#\[cfg(test)\]' "$file"; then
    if grep -Eq 'reqwest|ureq|TcpStream::connect|UdpSocket::bind|hyper::|surf::|isahc::' "$file"; then
      violations="$violations\n$file"
    fi
  fi
done

if [ -n "$violations" ]; then
  echo "network usage in unit tests is forbidden. files:$violations" >&2
  exit 1
fi

echo "no network calls in unit tests"
