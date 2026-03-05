# atlas-client

Python SDK for Atlas HTTP APIs.

## Publication policy

PyPI publication is deferred until release packaging is finalized. This crate directory is the canonical in-repo source for client packaging, verification, and documentation inputs.

## Quickstart

```bash
python -m pip install -e crates/bijux-atlas-client-python
python crates/bijux-atlas-client-python/examples/simple_query.py
```

## Documentation And Verification

Use `bijux-dev-atlas` for client docs generation and validation:

```bash
cargo run -p bijux-dev-atlas -- clients docs-generate --client atlas-client
cargo run -p bijux-dev-atlas -- clients verify --client atlas-client
```

## Features

- HTTP dataset query wrapper
- retry and timeout configuration
- pagination helpers
- logging and tracing hooks
