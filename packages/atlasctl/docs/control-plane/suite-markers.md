# Suite Markers

Suite markers are stable selectors used by the suite registry SSOT.

- `required`: release-grade required checks and gates.
- `ci`: CI lane coverage and parity checks.
- `local`: fast local development checks.
- `slow`: intentionally slower checks for deeper validation.

`atlasctl suite check` enforces this marker document and fails on drift.
