# Contracts schemas

- Owner: `platform`
- Purpose: hold canonical JSON schemas for control-plane outputs and contract reports.
- Consumers: `bijux-dev-atlas`, CI artifact validation, and generated report checks.
- Update workflow: update schema files with the producing command change, then rerun schema and contract validation.
