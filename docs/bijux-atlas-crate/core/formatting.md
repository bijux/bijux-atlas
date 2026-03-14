# Debug and Display Policy

Stability rules:
- `Display` output for shared error codes must remain deterministic.
- `Debug` is allowed for diagnostics but must avoid nondeterministic data.
- Public machine contracts must not parse `Debug` output.

Current contract:
- `ErrorCode::Display` is stable and maps to canonical machine code strings.
- `MachineError::Display` follows `"<code>: <message>"`.
