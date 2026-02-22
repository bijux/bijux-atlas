# How To Remove Bypasses Safely

Use this sequence to remove a bypass/allowlist entry without breaking unrelated workflows.

## Removal Flow

1. Identify the bypass entry and owning issue.
2. Confirm the original failure mode still exists (or already no longer reproduces).
3. Implement the underlying fix in code/config/docs (not another bypass).
4. Remove the bypass entry from its SSOT file.
5. Run the relevant local command/check lane (area-specific first, then broader policy checks).
6. Run bypass inventory/report commands and confirm count/trend moves down.
7. Update migration docs/milestones if this was a temporary migration bypass.

## Safety Checks

- Verify the bypass entry does not have hidden downstream dependencies.
- If the entry uses wildcards, verify exact matched files before removal.
- Prefer deleting one bypass per change when possible to simplify rollback.

## Useful Commands

- `./bin/atlasctl policies bypass inventory --report json`
- `./bin/atlasctl policies bypass report --report json --out artifacts/reports/atlasctl/policies-bypass-report.json`
- `./bin/atlasctl report bypass --json`
- `./bin/atlasctl report bypass culprits --json`
