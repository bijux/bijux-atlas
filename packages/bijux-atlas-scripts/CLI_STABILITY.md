# CLI Stability

Stable commands are guaranteed to keep names and core JSON fields across patch releases.

Stable:
- `bijux-atlas doctor`
- `bijux-atlas inventory ...`
- `bijux-atlas gates ...`
- `bijux-atlas ops ...`
- `bijux-atlas docs ...`
- `bijux-atlas configs ...`
- `bijux-atlas policies ...`
- `bijux-atlas k8s ...`
- `bijux-atlas stack ...`
- `bijux-atlas load ...`
- `bijux-atlas obs ...`
- `bijux-atlas report ...`
- `bijux-atlas version`
- `bijux-atlas env`
- `bijux-atlas self-check`

Experimental:
- `bijux-atlas compat ...`
- `bijux-atlas completion ...`

Policy:
- New top-level commands must be registered in `bijux_atlas_scripts.domain_cmd.registry`.
- Any breaking CLI change must update this file and `docs/_generated/cli.md` in the same commit.
