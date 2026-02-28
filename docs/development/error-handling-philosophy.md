# Error Handling Philosophy

- Owner: `platform`
- Type: `concept`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@d489531b`
- Reason to exist: document how checks classify and report failures.

## Principles

- Do not hide contract failures.
- Use explicit categories for usage, policy, and internal failures.
- Emit machine-readable context for triage automation.

## Verify Success

Failure output maps to stable exit code behavior and includes reproducible command context.

## What to Read Next

- [Control Plane](control-plane.md)
- [Debugging Locally](debugging-locally.md)
