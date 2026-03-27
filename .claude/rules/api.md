---
paths:
  - "crates/rusvel-api/**"
---

# API Crate Rules

- All department routes follow `/api/dept/{id}/*` pattern
- Use the shared SSE helper for streaming endpoints
- Handlers go in separate modules under `src/`
- Wrap sync DB I/O in `spawn_blocking`
- Validate SQL identifiers and guard file tool paths
- CORS is restricted — don't widen without justification
