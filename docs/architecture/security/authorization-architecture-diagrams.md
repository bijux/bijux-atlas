# Authorization Architecture Diagrams

## Authorization Components

```mermaid
flowchart LR
  Req[HTTP Request] --> Authn[Authentication Context]
  Authn --> Roles[Role Registry]
  Roles --> Perms[Permission Evaluator]
  Authn --> Policy[Route Policy Rules]
  Perms --> Decision[Authorization Decision]
  Policy --> Decision
  Decision --> Allow[Allow]
  Decision --> Deny[Deny + Audit + Counters]
```
