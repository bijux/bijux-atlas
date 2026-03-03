# Data Deletion Requests

Atlas does not currently implement an end-user deletion workflow.

## Current posture

- Operational data minimization is handled through bounded logs and redaction.
- Audit and security records are retained according to `configs/observability/retention.yaml`.
- Deletion requests must currently be handled by infrastructure operators at the storage layer.
