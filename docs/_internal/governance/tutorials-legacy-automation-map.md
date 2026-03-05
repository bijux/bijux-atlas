# Tutorials Legacy Automation Map

This document maps legacy tutorial script responsibilities to governed `bijux-dev-atlas tutorials` commands.

## Current State

- `tutorials/scripts/`: not present.
- `tutorials/tests/*.py`: not present.

## Command Mapping

- `ingest_example_dataset.sh` -> `bijux-dev-atlas tutorials real-data ingest --run-id <id>`
- `cleanup_tutorial_workspace.sh` -> `bijux-dev-atlas tutorials real-data clean-run --run-id <id>`
- `integrity_check.sh` -> `bijux-dev-atlas tutorials real-data doctor`
- `reproducibility_check.sh` -> `bijux-dev-atlas tutorials real-data export-evidence --run-id <id>`
- `package_example_datasets.sh` -> `bijux-dev-atlas tutorials dataset package`
- `build_tutorial_docs.sh` -> `bijux-dev-atlas tutorials build docs`

## Policy

No new `.sh` or `.py` automation source is allowed under `tutorials/` unless explicitly declared in `configs/tutorials/legacy-script-exceptions.json` with non-expired metadata.
