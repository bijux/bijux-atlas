# Documentation Merge Clusters

Owner: `docs-governance`  
Reason to exist: define consolidation units that collapse duplicate narratives into canonical pages.

1. Start surfaces: `docs/start-here.md` <= `docs/start-here.md`, `docs/root/start-here.md`, `docs/start-here.md`
2. Product identity: `docs/product/index.md` <= `docs/PROJECT_IDENTITY.md`, `docs/PROJECT_DESCRIPTION_SNIPPET.md`
3. Glossary: `docs/glossary.md` <= `docs/root/glossary.md`, `docs/operations/slo/GLOSSARY.md`, `docs/_internal/style/terms-glossary.md`
4. Governance style: `docs/_internal/governance/style-guide.md` <= `docs/style.md`, `docs/_internal/style/style-guide.md`
5. Review checklist: `docs/_internal/governance/docs-review-checklist.md` <= `docs/_internal/style/docs-review-checklist.md`
6. Removal policy: `docs/_internal/governance/docs-removal-policy.md` <= `docs/_internal/style/docs-removal-policy.md`
7. Decision templates: `docs/_internal/governance/decision-template.md` <= `docs/architecture/decision-template.md`, `docs/operations/slo/SLA_DECISION_ADR_TEMPLATE.md`
8. Architecture map: `docs/architecture/architecture-map.md` <= `docs/architecture/repo-overview.md`, `docs/architecture/system-graph.md`
9. Boundaries: `docs/architecture/boundaries.md` <= `docs/architecture/boundary-maps.md`, `docs/architecture/layering/index.md`
10. Effects: `docs/architecture/effects.md` <= `docs/architecture/scripting.md`, `docs/architecture/no-layer-fixups.md`
11. API errors: `docs/api/errors.md` narrative + `docs/reference/errors.md` table, absorb `docs/contracts/errors.md`
12. API compatibility: `docs/api/compatibility.md` <= `docs/contracts/compatibility.md`
13. Local operations workflow: `docs/operations/run-locally.md` <= `docs/operations/how-to-run-locally.md`, `docs/operations/full-stack-local.md`, `docs/operations/local-stack.md`
14. Deploy workflow: `docs/operations/deploy.md` <= `docs/operations/kubernetes.md`, `docs/operations/release-workflows.md`
15. Incident workflow: `docs/operations/incident-response.md` <= `docs/operations/runbook-template.md`, `docs/operations/policy-violation-triage.md`
16. Reference commands: `docs/reference/commands.md` <= `docs/operations/reference/commands.md`
17. Reference configs: `docs/reference/configs.md` <= `docs/root/CONFIGS_OVERVIEW.md`
18. Reference schemas: `docs/reference/schemas.md` <= `docs/root/SCHEMA_INDEX.md`
19. Make targets: `docs/reference/make-targets.md` <= `docs/development/make-targets.md`, `docs/_internal/generated/make-targets.md`
20. Development toolchain: `docs/development/tooling/rust-toolchain.md` <= `docs/development/cargo.md`, `docs/development/cargo-profiles-ssot.md`
21. Contributing: `docs/development/contributing.md` <= `docs/contract.md`, `docs/root/CONTRIBUTION_MODEL.md`
22. Repo layout: `docs/development/repo-layout.md` <= `docs/root/REPOSITORY_STRUCTURE.md`, `docs/development/repo-surface.md`
23. Policy placement: `docs/_internal/governance/index.md` <= policy pages currently under `_style`, `operations`, `development/ci`
24. Ops reference tables: `docs/reference/index.md` <= `docs/operations/reference/*.md`
25. Concept registry: `docs/_internal/generated/concept-registry.md` <= `docs/_internal/style/CONCEPT_REGISTRY.md`
26. Root mirrors: section `index.md` pages <= `docs/root/*_OVERVIEW.md` mirrors
27. Quickstart commands: `docs/operations/run-locally.md` <= `docs/quickstart/ops-local-full.md`
28. Client examples: `docs/api/index.md` + operations quickstart <= `docs/quickstart/client-sdk-examples.md`
29. Local cluster setup: `docs/operations/deploy.md` <= `docs/quickstart/local-cluster-setup.md`
30. Docs policy sources: `docs/_internal/governance/docs-operating-model.md` <= `docs/operations/DOCS_STRUCTURE_FREEZE.md`, `docs/operations/DOCS_CONVERGENCE_POLICY.md`
