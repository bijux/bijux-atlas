# Config Sources

`configs/sources/` holds human-authored configuration inputs that tools consume directly.

Rules:
- Put authoritative inputs here, grouped by the domain they govern.
- Keep registries out of this tree. Ownership, consumers, and inventories belong under `configs/registry/`.
- Keep schemas out of this tree. Validation contracts belong under `configs/schemas/`.
- Keep examples out of this tree. Non-authoritative examples belong under `configs/examples/`.

If a file is edited by hand and drives tool behavior, this is the default place to look first.
