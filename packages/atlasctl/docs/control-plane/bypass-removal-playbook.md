# Bypass Removal Playbook

Use this flow for any bypass entry.

1. Find the entry:
   - `./bin/atlasctl policies bypass entry --id <source:key>`
2. Confirm ownership and issue scope.
3. Implement the underlying fix.
4. Remove bypass record from its source file.
5. Run repo checks:
   - `./bin/atlasctl check run --group repo`
6. Regenerate any derived artifacts if needed.
7. Submit PR with note that bypass was removed.

## Definition of Done
- Entry removed from source policy file.
- Bypass report count does not increase.
- Repo checks stay green without adding new exceptions.
