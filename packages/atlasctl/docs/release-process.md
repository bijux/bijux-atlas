# Atlasctl Release Process

1. Run `atlasctl suite run all --json`.
2. Run `atlasctl suite run required_proof --json`.
3. Run `atlasctl suite check --json` and ensure suite inventory is clean.
4. Run `atlasctl contracts validate --report json`.
5. Run `atlasctl docs validate --report json`.
6. Refresh goldens only with `atlasctl gen goldens`, then review diffs.
7. Run `atlasctl internal legacy inventory --report json` and confirm `count=0`.
8. Update `docs/release-notes.md` and tag only after all gates pass.
