# Snapshot Test Policy

Snapshot tests must be deterministic.

Rules:

- No snapshots dependent on wall clock time, locale, random seeds, hostnames, env vars, or filesystem ordering.
- Inputs must be fixed fixtures committed in-repo.
- Output serialization must use stable ordering.
- Snapshot updates must include rationale in PR description.
- CI must run snapshots in a hermetic environment.
