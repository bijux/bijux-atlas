# Migration Status

`atlasctl` is the single scripting control plane.  
Use machine output from `atlasctl migration status --json` as the source for migration tracking.

## Commands

- `atlasctl inventory legacy-scripts --format json --dry-run`
- `atlasctl inventory commands --format json --dry-run`
- `atlasctl migration status --json`
- `atlasctl migration gate --json`
- `atlasctl migration diff --json`
- `atlasctl inventory touched-paths --command <name> --format json --dry-run`

## Policy

- Migration gate enforces remaining-script budget using `configs/policy/migration_exceptions.json`.
- Any expired migration exception fails the gate.
- Docs and Makefiles must not reference `scripts/` paths.
