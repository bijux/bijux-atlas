# Bypass Removal Playbook

Use this flow for any bypass entry. It matches the structured ops bypass ledger
(`ops/_meta/bypass-ledger.json`) and structured ops allowlists
(`ops/_meta/*allowlist.json`).

1. Find the entry:
   - `./bin/atlasctl policies bypass entry --id <source:key>`
   - `./bin/atlasctl policies culprits --format json --blame`
2. Confirm ledger metadata is complete (owner, expires_at, task_id, replacement_mechanism, domain).
3. Implement the underlying fix.
4. Remove bypass record from its source file (and ledger entry if fully removed).
5. Run repo checks:
   - `./bin/atlasctl check run --group repo`
6. Regenerate any derived artifacts if needed.
7. Update burn-down outputs and verify trend/count moved in the right direction:
   - `./bin/atlasctl policies bypass burn-down --report json --update-trend`
   - `./bin/atlasctl report bypass --json`
8. Generate PR checklist snippet and submit PR:
   - `./bin/atlasctl policies bypass pr-checklist --report text`

## Definition of Done
- Entry removed from source policy file.
- Ledger metadata removed or updated consistently.
- Bypass report count does not increase.
- Repo checks stay green without adding new exceptions.
