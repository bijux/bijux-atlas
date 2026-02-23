# Ops Command Group

Stable public `atlasctl ops ...` command surface.

## Canonical Areas

- `stack`
- `deploy`
- `k8s`
- `obs`
- `load`
- `e2e`
- `datasets`
- `pins`
- `reports`

## Stable Subcommands (Public)

- `atlasctl ops stack {check,verify,report,up,down,status,validate}`
- `atlasctl ops deploy {check,verify,report,plan,apply,rollback}`
- `atlasctl ops k8s {check,verify,report,render,validate,diff}`
- `atlasctl ops obs {check,verify,report,lint,drill}`
- `atlasctl ops load {check,verify,report,run,compare}`
- `atlasctl ops e2e {check,verify,report,run,validate-results}`
- `atlasctl ops datasets {check,verify,report,lock,qc,validate}`
- `atlasctl ops pins {check,verify,report}`

## Public vs Internal

- `commands/ops/internal/**` is implementation-only and must not appear in public help/docs.
- Temporary migration glue must use `commands/ops/internal/migrate_*`.

## Examples

- `atlasctl ops --report json stack status`
- `atlasctl ops --report json deploy plan`
- `atlasctl ops --report json k8s render`
- `atlasctl ops --report json load compare`
