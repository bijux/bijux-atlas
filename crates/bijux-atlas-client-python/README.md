# atlas-client

Python SDK for Atlas HTTP APIs.

## Quickstart

```bash
python -m pip install -e clients/atlas-client
python clients/atlas-client/examples/simple_query.py
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
