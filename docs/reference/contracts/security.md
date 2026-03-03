# Security Contracts

- Owner: `bijux-atlas-security`
- Type: `reference`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@bb6ad845da4ad296a761a62e4b69f40969f7b563`
- Reason to exist: define the machine-readable security contract checks and their sources.

## Threat model checks

- `SEC-THREAT-001`: threat model files validate and are complete.
- `SEC-THREAT-002`: every threat has at least one mitigation.
- `SEC-THREAT-003`: every mitigation maps to a concrete control or documented reason.
- `SEC-THREAT-004`: high severity threats map to an executable check or explicit runbook.

## Compliance checks

- `SEC-COMP-001`: compliance matrix validates.
- `SEC-COMP-002`: every control has at least one evidence pointer.
- `SEC-COMP-003`: evidence pointers resolve to existing files.

## Secret handling checks

- `SEC-RED-001`: redaction policy covers all declared secrets.
- `SEC-RED-002`: the default release evidence directory contains no declared secret matches.
- `SEC-ART-001`: artifact scan passes for the selected directory.

## Source files

- Threat model: `security/threat-model/*.md`, `security/threat-model/*.yaml`
- Compliance: `security/compliance/*.yaml`
- Secret policy: `configs/security/*.json`
- Schemas: `configs/contracts/security/*.json`
