# Security

- Owner: `bijux-atlas-security`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Reason to exist: provide security operations entrypoints.

## Start here

- [Security Model Scope](model-scope.md)
- [Deploy Behind Auth Proxy](deploy-behind-auth-proxy.md)
- [Security Posture](../security-posture.md)
- [Security Compliance](compliance.md)
- [API Access Model](api-access-model.md)
- [API Key Usage](api-key-usage.md)
- [Token Authentication Flow](token-authentication-flow.md)
- [Authentication Troubleshooting](authentication-troubleshooting.md)
- [Authentication Examples](authentication-examples.md)
- [Authorization Reference](authorization-reference.md)
- [Dataset Access Policies](dataset-access-policies.md)
- [Administrative Access Model](administrative-access-model.md)
- [Authorization Troubleshooting](authorization-troubleshooting.md)
- [Authorization Examples](authorization-examples.md)
- [Role Configuration Examples](role-configuration-examples.md)
- [Data Classification Model](data-classification-model.md)
- [Sensitive Data Handling Rules](sensitive-data-handling-rules.md)
- [Data Retention Policy](data-retention-policy.md)
- [TLS Configuration](tls-configuration.md)
- [Artifact Integrity Verification](artifact-integrity-verification.md)
- [Dataset Security Guarantees](dataset-security-guarantees.md)
- [Secure Deployment Guidelines](secure-deployment-guidelines.md)
- [TLS Configuration Examples](tls-configuration-examples.md)
- [Security Deployment Examples](security-deployment-examples.md)
- [Data Protection Best Practices](data-protection-best-practices.md)
- [Data Protection Summary Report](data-protection-summary-report.md)
- [Security Testing And Monitoring](security-testing-and-monitoring.md)
- [Vulnerability Scanning Policy](vulnerability-scanning-policy.md)
- [Dependency Security Scanning](dependency-security-scanning.md)
- [Dependency Version Monitoring](dependency-version-monitoring.md)
- [Dependency Risk Scoring](dependency-risk-scoring.md)
- [Static Analysis Security Rules](static-analysis-security-rules.md)
- [Runtime Security Monitoring](runtime-security-monitoring.md)
- [Secure Development Practices](secure-development-practices.md)
- [Vulnerability Reporting Policy](vulnerability-reporting-policy.md)
- [Incident Response Process](incident-response-process.md)
- [Security Patch Policy](security-patch-policy.md)
- [Security Release Checklist](security-release-checklist.md)
- [Security Runbooks](security-runbooks.md)
- [Security Troubleshooting Guide](security-troubleshooting-guide.md)
- [Security FAQ](security-faq.md)
- [Security Glossary](security-glossary.md)
- [Security Roadmap](security-roadmap.md)
- [Security Readiness Checklist](security-readiness-checklist.md)
- [Security Completion Report](security-completion-report.md)
- [Audit And Security Event Classification](audit-and-security-event-classification.md)
- [Auth Error Codes](auth-error-codes.md)
- [Audit Logging Model](audit-logging.md)
- [Enable Audit Logging And Retention](enable-audit-logging.md)
- [Audit Log Rotation And Export](audit-log-rotation.md)
- [Audit Log Access](audit-log-access.md)
- [Audit Log Field Inventory](log-field-inventory.md)
- [Safe Logging Guidelines](safe-logging-guidelines.md)
- [Respond To Suspicious Activity](respond-to-suspicious-activity.md)
- [Export Audit Logs To Institutional SIEM](export-audit-logs-to-siem.md)
- [Security Incident Response](incident-response.md)
- [Privacy Stance](privacy-stance.md)
- [Data Deletion Requests](data-deletion-requests.md)
- [Security Key Rotation](key-rotation.md)
- [Refresh Pinned GitHub Actions SHAs](action-pin-refresh.md)
- [Security Review Checklist](review-checklist.md)
- [Advisory Process](advisory-process.md)
- [Incident Response](../incident-response.md)

## Verify success

Security workflows are successful when posture checks pass and incident handling remains auditable.

## Threat model outline

- Assets: dataset artifacts, runtime credentials, release bundles, and audit logs.
- Trust boundaries: API edge, control-plane execution, artifact storage, and cluster runtime.
- Primary risks: unauthorized access, tampering, credential leakage, and audit gaps.
- Controls: authentication, authorization, encryption, integrity checks, and incident response.

## Next

- [Release Operations](../release/index.md)

- [Security Philosophy](security-philosophy.md)
- [Threat Modeling Methodology](threat-modeling-methodology.md)
- [Security Architecture Overview](security-architecture-overview.md)
- [Trust Boundary Diagram](trust-boundary-diagram.md)
- [Attacker Capability Model](attacker-capability-model.md)
- [Attack Surface Inventory](attack-surface-inventory.md)
- [Privileged Components](privileged-components.md)
- [Sensitive Data Flows](sensitive-data-flows.md)
- [Artifact Integrity Attack Vectors](artifact-integrity-attack-vectors.md)
- [Configuration Injection Risks](configuration-injection-risks.md)
- [Threat Model Reference](threat-model-reference.md)
- [Threat Model Diagram](threat-model-diagram.md)
- [Threat Classification Taxonomy](threat-classification-taxonomy.md)
- [Threat Registry Reference](threat-registry-reference.md)
- [Threat Model Coverage Report](threat-model-coverage-report.md)
- [Threat Model](threat-model.md)
- [Mitigation Strategy](mitigation-strategy.md)
- [Residual Risk Register](residual-risk-register.md)
- [Security Monitoring Strategy](security-monitoring-strategy.md)
- [Incident Classification Policy](incident-classification-policy.md)
- [Vulnerability Disclosure Policy](vulnerability-disclosure-policy.md)
- [Security Architecture Documentation](security-architecture-documentation.md)
- [Threat Model Troubleshooting](threat-model-troubleshooting.md)
- [Threat Model Review Checklist](threat-model-review-checklist.md)
- [Threat Model Update Policy](threat-model-update-policy.md)
- [Threat Model CI Validation](threat-model-ci-validation.md)
- [Threat Model Audit Scenario](threat-model-audit-scenario.md)
- [Threat Modeling Workshop Guide](threat-model-workshop-guide.md)
- [Threat Modeling Metrics](threat-model-metrics.md)
- [Threat Modeling Delivery Report](threat-model-delivery-report.md)