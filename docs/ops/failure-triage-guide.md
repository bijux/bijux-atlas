# Failure Triage Guide

- Owner: bijux-atlas-operations
- Stability: stable

Triage order:
1. Validate command and run-id context.
2. Inspect `artifacts/<run-id>/reports`.
3. Inspect `artifacts/<run-id>/logs`.
4. Re-run failing lane with JSON output enabled.
