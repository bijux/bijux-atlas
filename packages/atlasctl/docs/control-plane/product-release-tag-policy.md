# Product Release Tag Policy

Release tags must follow the canonical product version pattern:

- `vMAJOR.MINOR.PATCH`
- optional prerelease/build suffixes are allowed:
  - `v1.2.3-rc.1`
  - `v1.2.3+build.7`

`atlasctl product release-candidate` enforces this contract when a tag is present via `GITHUB_REF_NAME` or `RELEASE_TAG`.

This policy exists so release automation, artifact manifests, and release verification lanes can parse version intent deterministically.
