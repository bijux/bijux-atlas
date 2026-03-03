# Error Handling Philosophy

- Owner: `platform`
- Type: `concept`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@240605bb1dd034f0f58f07a313d49d280f81556c`
- Reason to exist: document how checks classify and report failures.

## Principles

- Do not hide contract failures.
- Use explicit categories for usage, policy, and internal failures.
- Emit machine-readable context for triage automation.

## Verify Success

Failure output maps to stable exit code behavior and includes reproducible command context.

## What to Read Next

- [Control-plane](../control-plane/index.md)
- [Debugging Locally](debugging-locally.md)
