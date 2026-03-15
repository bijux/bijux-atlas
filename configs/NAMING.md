# Config Naming Rules

`configs/` uses path names as part of the contract surface, so names need to stay boring and durable.

Rules:
- Use directory names to show role before detail. `registry/` stores metadata, `schemas/` stores validation contracts, and domain inputs live under dedicated domain directories.
- Prefer kebab-case for new files and directories.
- Keep filenames specific to the thing they govern. Prefer names that describe the governed surface directly instead of numbered or milestone-style labels.
- Use singular names for a single policy or registry and plural names only when the file is a collection.
- Keep path changes rare. Moving or renaming a config path is a contract change and needs the same care as changing a command or API surface.

When a name is unclear, prefer the path that tells a new maintainer what the file governs without opening it.
