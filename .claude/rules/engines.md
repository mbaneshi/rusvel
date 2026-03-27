---
paths:
  - "crates/*-engine/**"
---

# Engine Crate Rules

- Engines depend ONLY on `rusvel-core` port traits — never import adapter crates
- Use `AgentPort` for LLM interaction, never `LlmPort` directly (ADR-009)
- All domain structs must have `metadata: serde_json::Value` (ADR-007)
- Event kinds are `String` constants, not enums (ADR-005)
- Keep crate under 2000 lines total
- Use `thiserror` for error types
- Human approval gates required for content publishing and outreach (ADR-008)
