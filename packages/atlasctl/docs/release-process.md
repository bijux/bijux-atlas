# Atlasctl Release Process

1. Run `atlasctl suite run refgrade_proof --json`.
2. Run `atlasctl suite check --json` and ensure suite inventory is clean.
3. Run `atlasctl contracts validate --report json`.
4. Run `atlasctl docs validate --report json`.
5. Refresh goldens only with `atlasctl gen goldens`, then review diffs.
6. Update release notes/changelog and tag only after all gates pass.
