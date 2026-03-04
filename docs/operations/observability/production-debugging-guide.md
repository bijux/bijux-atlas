# Production Debugging Guide

## Safe debug workflow

1. Keep debug endpoint access restricted to trusted operators.
2. Use `bijux-dev-atlas system debug ...` commands to keep output structure stable.
3. Prefer snapshot capture over repeated live polling during incidents.
4. Record command, UTC timestamp, and incident id with each artifact export.

## Required captures

- `system debug health-checks`
- `system debug diagnostics`
- `system debug runtime-state`
- `system debug metrics-snapshot`
