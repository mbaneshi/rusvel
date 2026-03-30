# ADR-015: Agent Intelligence Layer — Reasoning Framework, Verification Loop, Parallel Tools

**Date:** 2026-03-30
**Status:** Proposed
**Supersedes:** None
**Extends:** ADR-009 (AgentPort), ADR-014 (DepartmentApp)

---

## Context

RUSVEL's agent runtime (`rusvel-agent`) implements a sound ReAct loop with streaming, deferred tool loading, consecutive-failure reflection (3-strike system), and configurable iteration limits (default 50). However, agents produce significantly lower-quality outputs than equivalent interactions through Claude Code. Root cause analysis identifies six specific gaps:

1. **System prompts lack reasoning framework.** Agents are told WHO they are (~200 tokens) but not HOW to think. No guidance on search-before-write, tool selection, error recovery, or output standards. Claude Code uses 5000+ tokens of structured reasoning instructions.

2. **Single tool call per iteration.** `extract_tool_call()` (lib.rs:339) returns only the first `Part::ToolCall`. When Claude returns 3 parallel reads, only 1 executes per iteration, tripling latency.

3. **Verification chain exists but is unwired.** `verification.rs` has `LlmCritiqueStep` and `RulesComplianceStep` with `suggested_fix` support. The chain is never called from `run_streaming_loop`. Self-correction code lies dormant.

4. **No graceful degradation at iteration limit.** When `max_iterations` is reached (lib.rs:851), the function returns `Err`. Partial work is lost.

5. **DepartmentManifest has no reasoning guidance.** Each department declares `system_prompt`, `capabilities`, `tools` — but no structured reasoning framework or compaction preservation rules.

6. **No permission mode.** No plan/dry-run mode. No formal read-only vs stateful tool classification.

## Decision

Introduce an **Agent Intelligence Layer** with six changes across three crates.

### Change 1: Core Reasoning Framework

Add `crates/rusvel-agent/src/prompts/core_framework.md` (~800 tokens), loaded via `include_str!`. Prepended to every agent system prompt. Content: search-before-write, tool selection matrix, error recovery strategy, output quality standards, safety constraints.

Add per-department guidance via `DepartmentManifest::reasoning_guidance` field.

### Change 2: Parallel Tool Execution

Replace `extract_tool_call()` → `extract_all_tool_calls()`. Execute tools with `read_only: true` metadata concurrently via `tokio::JoinSet`, stateful tools sequentially. Emit `ToolCall`/`ToolResult` events for each.

### Change 3: Verification Integration

Add `verification_chain: Option<Arc<VerificationChain>>` to `AgentRuntime`. After `FinishReason::Stop`, run the chain. On `Fail`, inject `suggested_fix` as user message and continue loop (up to `max_verification_retries` on `AgentConfig`, default 1).

### Change 4: Graceful Degradation

At iteration `max - 1`, inject system message: "Summarize accomplishments and remaining work." Final LLM call with empty tools. Return summary, not error.

### Change 5: DepartmentManifest Extensions

```rust
// Two new fields on DepartmentManifest
pub reasoning_guidance: String,          // Department-specific reasoning
pub compaction_preserve: Vec<String>,    // What to keep during summarization
```

### Change 6: Permission Mode

```rust
pub enum PermissionMode {
    Default,       // Normal execution, pre-hooks can deny
    Plan,          // No tool execution — describe what would happen
    Supervised,    // All tool calls emit ApprovalRequired event
}
```

Added to `AgentConfig`. In `Plan` mode, tools return descriptions instead of executing.

## Consequences

- Agent quality improves without changing loop architecture
- Parallel tool calls reduce latency 2-3x for read-heavy tasks
- Self-correction catches errors before reaching user
- No "max iterations exceeded" errors — always returns useful output
- All changes additive — no breaking API changes
- `rusvel-core`: 2 new fields on `DepartmentManifest`, 1 new enum, 1 new field on `AgentConfig`
- `rusvel-agent`: internal improvements, no port trait changes
- Verification chain tests already exist; extend with integration tests
