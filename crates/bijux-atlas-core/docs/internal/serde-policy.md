# Serialization Policy

Rules:
- Use serde for wire/manifest compatibility only.
- Stable fields are explicit and unknown fields are rejected where contract requires strictness.
- Machine-facing maps use deterministic key ordering (`BTreeMap`).

Current contract:
- `MachineError` is serde-serializable and `deny_unknown_fields` is enabled.
