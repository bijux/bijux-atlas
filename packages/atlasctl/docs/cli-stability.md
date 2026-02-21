# CLI Stability

Stable commands are guaranteed to keep names and core JSON fields across patch releases.

Stable:
- `atlasctl doctor`
- `atlasctl inventory ...`
- `atlasctl gates ...`
- `atlasctl ops ...`
- `atlasctl docs ...`
- `atlasctl configs ...`
- `atlasctl policies ...`
- `atlasctl k8s ...`
- `atlasctl stack ...`
- `atlasctl load ...`
- `atlasctl obs ...`
- `atlasctl report ...`
- `atlasctl version`
- `atlasctl env`
- `atlasctl paths`
- `atlasctl self-check`

Experimental:
- `atlasctl completion ...`

Exit codes:
- `0`: ok
- `2`: user/config/input error
- `3`: contract/schema/validation failure
- `10`: missing prereq/tooling dependency
- `20`: internal/unexpected failure

Policy:
- New top-level commands must be registered in `atlasctl.domain_cmd.registry`.
- Any breaking CLI change must update this file and `docs/_generated/cli.md` in the same commit.
