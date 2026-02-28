# Run Locally

Owner: `bijux-atlas-operations`  
Audience: `operator`, `contributor`  
Type: `runbook`  
Reason to exist: provide one canonical local execution workflow.

## Prerequisites

- Container runtime available.
- Local tooling installed for Atlas workflows.
- Access to fixture datasets.

## Workflow

1. Start local services.
2. Ingest fixture dataset.
3. Run one API smoke query.
4. Verify health and metrics.
5. Stop services and clean temporary state.

## Canonical Details

- [How To Run Locally](how-to-run-locally.md)
- [Full Stack Local](full-stack-local.md)
- [Local Stack](local-stack.md)
- [Fixture Dataset Ingest](fixture-dataset-ingest.md)
