# Administrative Access Model

- Owner: `bijux-atlas-security`
- Type: `policy`
- Audience: `operator`
- Stability: `stable`
- Reason to exist: define admin authorization controls.

## Administrative Surface

- `/debug/*`
- `/v1/_debug/*`

## Required Permission

Administrative requests require:

- action: `ops.admin`
- resource kind: `namespace`

## Principal Allowlist

- `operator`
- `ci` (controlled automation only)

## Denial Handling

Authorization denials emit:

- structured event `authorization_denied`
- audit event `authorization_denied` when audit logging is enabled
- denial counter in policy violation metrics
