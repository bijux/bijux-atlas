#!/usr/bin/env bash
set -euo pipefail

: "${RELEASE_TAG:?RELEASE_TAG is required}"
: "${RELEASE_VERSION:?RELEASE_VERSION is required}"

make dist

assets_dir="artifacts/github-release"
rm -rf "${assets_dir}"
mkdir -p "${assets_dir}"

cp artifacts/dist/release/sha256sum.txt "${assets_dir}/sha256sums.txt"
cp ops/release/crates-release.toml "${assets_dir}/crates-release.toml"
cp ops/release/images-release.toml "${assets_dir}/images-release.toml"
cp ops/release/ops-release.toml "${assets_dir}/ops-release.toml"

cat > "${assets_dir}/release-notes.md" <<NOTES
Atlas release ${RELEASE_TAG}

Release version: ${RELEASE_VERSION}
Tag: ${RELEASE_TAG}

Published package surfaces:
- crates.io: https://crates.io/crates/bijux-atlas/${RELEASE_VERSION}
- docs site: https://bijux.io/bijux-atlas/
- Rust docs: https://docs.rs/bijux-atlas/latest/bijux_atlas/

Attached assets:
- release tarball built from \`make dist\`
- SHA-256 checksums for the attached files
- stable release specifications for crates, images, and ops packaging
NOTES
