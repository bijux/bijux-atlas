# Layering Rules

`bijux-dev-atlas` uses explicit layers with one-way dependencies.

Allowed direction:

- `engine -> model`
- `engine -> runtime`
- `domains -> engine`
- `domains -> model`
- `domains -> runtime`
- `cli -> engine`
- `cli -> registry`
- `cli -> ui`

Disallowed direction:

- `model` must not import `runtime`, `commands`, or `cli`
- `engine` must not import `commands` or `cli`
- domain implementations must not import `cli`

Enforcement:

- `crates/bijux-dev-atlas/tests/layering_enforcement.rs` scans Rust sources for forbidden imports
- `.github/workflows/layering-boundaries.yml` runs the layering tests on pull requests and merge
  groups
- merge protection should require the `layering-boundaries` check
