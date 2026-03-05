# Trace Schema Reference

- Owner: `bijux-atlas-operations`
- Type: `reference`
- Audience: `operator`
- Stability: `stable`
- Reason to exist: define the schema contract for trace verification and coverage reports.

## Trace Contract Fields

- `schema_version`: schema contract version
- `architecture`: high-level tracing architecture description
- `context_propagation_policy`: propagation requirements
- `span_naming_convention`: canonical naming rules
- `sampling_strategy`: deterministic sampling strategy
- `retention_policy`: retention requirements
- `span_registry`: list of required span definitions

## Span Entry Fields

- `id`: stable trace identifier
- `span_name`: canonical span name
- `layer`: subsystem classification
- `summary`: operator-facing summary
- `required_attributes`: mandatory attributes for integrity
- `async_propagation_required`: async propagation requirement
- `request_id_correlation`: request correlation requirement
- `logging_correlation`: log correlation requirement
- `metrics_correlation`: metric correlation requirement
- `stability`: contract stability level

## Verification Report Schemas

`observe traces verify` output:

- `kind`: `observe_traces_verify`
- `status`: `ok` or `failed`
- `violations`: ordered integrity violations
- `artifacts`: generated report and diagram paths

`observe traces coverage` output:

- `kind`: `observe_traces_coverage`
- `report.coverage`: required span coverage rows
- `report.summary`: coverage totals and ratio

`observe traces topology` output:

- `kind`: `observe_traces_topology`
- `artifact`: generated mermaid diagram path
