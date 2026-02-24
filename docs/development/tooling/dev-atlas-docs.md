# Dev Atlas Docs Commands

`bijux dev atlas docs` is the canonical docs control-plane surface for local validation and CI docs checks.

## Commands

- `bijux dev atlas docs doctor --format json`
- `bijux dev atlas docs validate --format json`
- `bijux dev atlas docs links --format json`
- `bijux dev atlas docs lint --format json`
- `bijux dev atlas docs inventory --format json`
- `bijux dev atlas docs grep <pattern> --format json`
- `bijux dev atlas docs build --allow-subprocess --allow-write --format json`
- `bijux dev atlas docs serve --allow-subprocess --format text`

## CI Usage

- Safe CI lane (no subprocess): `bijux dev atlas docs doctor --format json`
- Optional build lane: `bijux dev atlas docs build --allow-subprocess --allow-write --format json`

## Make Wrappers

- `make docs` delegates to `bijux dev atlas docs doctor`
- `make docs-serve` delegates to `bijux dev atlas docs serve`
