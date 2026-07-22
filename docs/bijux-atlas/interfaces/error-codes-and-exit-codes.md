---
title: Error Codes and Exit Codes
audience: mixed
type: reference
status: canonical
owner: atlas-docs
last_reviewed: 2026-07-22
---

# Error Codes and Exit Codes

Atlas uses structured error reporting to keep failures classifiable and
automatable.

## Error Reporting Model

```mermaid
flowchart LR
    Failure[Failure] --> ApiError[API error code]
    Failure --> ExitCode[CLI exit behavior]
    ApiError --> Client[Client handling]
    ExitCode --> Automation[Automation handling]
```

This error-reporting model separates two important consumers: HTTP clients and
CLI automation. Both need structured signals, but they consume them through
different surfaces.

## HTTP Error Envelope

```json
{
  "error": {
    "code": "DatasetNotFound",
    "message": "dataset not found",
    "details": {},
    "request_id": "req-..."
  }
}
```

`code` is the stable branch key. `message` and `details` explain the specific
failure. `request_id` joins the response to logs and traces. Clients must not
infer a code by matching message text.

## HTTP Status and Code Families

| Status | Codes observed through that mapping | Client interpretation |
| ---: | --- | --- |
| `400` | `InvalidQueryParameter`, `InvalidCursor`, `MissingDatasetDimension`, `RangeTooLarge`, `ValidationFailed` | Correct request shape, cursor, or dataset identity before retrying. |
| `401` | `AuthenticationRequired` | Supply valid authentication under the deployment policy. |
| `403` | `AccessForbidden` | The authenticated principal is not authorized for the action. |
| `404` | `DatasetNotFound`, `GeneNotFound` | Confirm dataset discovery and requested identifier. |
| `413` | `PayloadTooLarge`, `ResponseTooLarge` | Reduce request or response scope. |
| `422` | `QueryRejectedByPolicy`, `QueryTooExpensive` | Change the request or governing policy; an identical retry is not corrective. |
| `429` | `RateLimited` | Respect `Retry-After` and retry only when the operation is safe to repeat. |
| `409` | `ArtifactCorrupted`, `ArtifactQuarantined` on dataset-open paths | Stop using the affected artifact and preserve quarantine or corruption evidence. |
| `503` | `NotReady`, `UpstreamStoreUnavailable`, and `Timeout` through the shared default mapping | Respect `Retry-After`, then re-evaluate readiness and dependency state. |
| `504` | `Timeout` on the gene request deadline path | Treat the request as timed out; do not assume backend unavailability. |
| `500` | `Internal`, integrity or quarantine codes through the default mapping, or another unmapped internal failure | Preserve the request ID and escalate; do not hide it with unlimited retries. |

The shared code registry also contains ingest validation codes. Their presence
in the type and OpenAPI enum does not mean every runtime endpoint can emit each
one. The generated registry and endpoint response contracts remain the exact
authority.

HTTP status expresses the endpoint's transport decision; the stable code
expresses the failure category. The mapping is therefore not globally
one-to-one. `Timeout` can appear as the shared `503` mapping or as `504` on the
gene request deadline, while artifact integrity codes can be `409` on dataset
open paths even though the shared fallback maps them to `500`.

```mermaid
flowchart LR
    Cause["domain or dependency failure"] --> Code["stable error code"]
    Code --> Endpoint["endpoint context"]
    Endpoint --> Status["HTTP status + headers"]
    Status --> Policy["client retry or correction policy"]
    Code --> Correlate["request ID, logs, and traces"]
```

Clients should branch on both code and status, retain unknown codes, and apply
retry only when the operation itself is safe to repeat. A new endpoint mapping
must not be inferred from another endpoint that happens to emit the same code.

## Retry Decision

```mermaid
flowchart TD
    Failure[HTTP failure] --> Code[Read status and stable code]
    Code --> Correctable{Request is incorrect?}
    Correctable -->|yes| Change[Correct request; do not blind retry]
    Correctable -->|no| Transient{429 or retryable 503?}
    Transient -->|yes| Bound[Honor Retry-After and bounded retry policy]
    Transient -->|no| Preserve[Preserve request ID and escalate or surface]
```

Atlas currently attaches `Retry-After: 3` to `429` and `503` error responses.
That header is a lower-bound hint, not proof that the cause will clear or that
an arbitrary mutation is idempotent. Cap attempts, add jitter in shared client
fleets, and retain the terminal response.

## CLI Exit Codes

| Exit | Name | Meaning |
| ---: | --- | --- |
| `0` | `success` | Command completed successfully. |
| `2` | `usage` | Command syntax, argument, or namespace is invalid. |
| `3` | `validation` | Input, policy, evidence, immutability, or contract validation failed. |
| `4` | `dependency_failure` | A required file, database, network path, or external dependency failed. |
| `10` | `internal` | The command could not classify or complete an internal operation. |

In JSON mode, the CLI emits a machine error with `code`, `message`, and a
string map of `details`. Shell automation should branch on the numeric exit
class, refine handling with the machine code, and retain stderr or JSON detail
for diagnosis. An internal exit must remain visible; converting it to success
because a report file exists destroys the failure contract.

The current product CLI classifies many action failures into validation,
dependency, or internal exits by inspecting normalized error text. This is a
compatibility-sensitive boundary: changing producer wording can change the
numeric class even when the underlying cause is unchanged. Preserve the full
machine error in automation, and treat changes to classification phrases as
exit-contract changes until action failures carry a typed category end to end.
