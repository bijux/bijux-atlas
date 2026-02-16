# Migration Notes

Future schema/version migration requirements:
- Introduce explicit schema version fields before behavioral changes.
- Document old/new field semantics and defaulting strategy.
- Keep deterministic serialization and ordering guarantees.
