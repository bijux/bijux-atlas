# Checks Taxonomy

- Owner: `team:atlas-governance`
- Type: `policy`
- Audience: `contributor`
- Stability: `stable`
- Reason to exist: define durable check classification and severity semantics.

## Execution Modes

- `static`: deterministic read-only checks.
- `effect`: checks that require bounded side effects.
- `integration`: checks that compose multiple governed surfaces.

Legacy `pure` mode in `checks.registry.json` maps to `static`.

## Severity Levels

- `blocker`: merge must not proceed.
- `major`: remediation required before release.
- `minor`: remediation planned in normal backlog.
- `info`: non-blocking signal.

## Required Metadata

Each registry entry must declare:

- stable `check_id`
- `owner`
- `mode`
- `severity`
- deterministic `commands`
- stable `report_ids`
- stable `reports`

## Authority

Policy authority mapping is stored in `configs/governance/check-policy-authority-map.json`.
