---
name: engine-check
description: Verify an engine crate follows hexagonal architecture rules. Use to check boundary compliance.
allowed-tools: Read, Grep, Glob, Bash
context: fork
agent: Explore
---

Check the engine crate `$ARGUMENTS` for architecture compliance.

Verify:
1. **No adapter imports** — `Cargo.toml` must not depend on `rusvel-db`, `rusvel-llm`, `rusvel-agent`, or any adapter crate
2. **Only port traits** — engine should import from `rusvel-core` only
3. **AgentPort not LlmPort** — grep for `LlmPort` usage (ADR-009 violation)
4. **metadata field** — all domain structs must have `metadata: serde_json::Value` (ADR-007)
5. **String event kinds** — no enum-based event kinds (ADR-005)
6. **Crate size** — total lines under 2000

Report pass/fail for each check with file:line references for any violations.
