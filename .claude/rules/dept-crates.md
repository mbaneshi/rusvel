---
paths:
  - "crates/dept-*/**"
---

# Department Wrapper Crate Rules

- Implement `DepartmentApp` trait from `rusvel-core`
- Provide `DepartmentManifest` with id, name, description, capabilities
- Depend on the corresponding `*-engine` crate + `rusvel-core`
- Keep thin — delegate logic to the engine, don't add business logic here
- Register in composition root (`rusvel-app`)
