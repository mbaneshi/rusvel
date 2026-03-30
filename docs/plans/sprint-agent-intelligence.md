# Sprint: Agent Intelligence Layer (ADR-015)

> **Goal:** Close the gap between Claude Code quality and RUSVEL agent quality.
> **ADR:** [ADR-015](../design/adr-015-agent-intelligence-layer.md)
> **Design:** [implementation-adr-015.md](implementation-adr-015.md)
> **Minibook:** [../proposals/agent-intelligence-minibook.md](../proposals/agent-intelligence-minibook.md)
> **Date:** 2026-03-30 | **Status:** Planned
> **Prerequisite:** Sprints 0-1 done.

---

## Current State (verified 2026-03-30)

| What | Status | File:Line |
|------|--------|-----------|
| `DEFAULT_MAX_ITERATIONS` | 50 (good) | `rusvel-agent/src/lib.rs:45` |
| `max_iterations` on AgentConfig | Exists | `rusvel-core/src/domain.rs:455` |
| Consecutive failure reflection | Exists (3-strike) | `rusvel-agent/src/lib.rs:826-841` |
| Tool descriptions WHEN TO USE | Exists for file ops | `rusvel-builtin-tools/src/file_ops.rs:36+` |
| VerificationChain | Exists, tested, **NOT wired** | `rusvel-agent/src/verification.rs` |
| `extract_tool_call` | Returns **first only** | `rusvel-agent/src/lib.rs:339-346` |
| DepartmentManifest | Rich, no reasoning fields | `rusvel-core/src/department/manifest.rs` |
| Graceful degradation | Missing — returns `Err` | `rusvel-agent/src/lib.rs:851-853` |
| Permission mode | Missing | — |
| Core reasoning framework | Missing | — |

---

## Sprint Tasks

### Phase A: Core Reasoning Framework (2 days)

| # | Task | Effort | Depends | Files |
|---|------|--------|---------|-------|
| A1 | **Core reasoning framework** — create `prompts/core_framework.md`, prepend to all agent system prompts | 4h | — | `rusvel-agent/src/prompts/core_framework.md` (new), `rusvel-agent/src/lib.rs` |
| A2 | **Department guidance prompts** — create `dept_{id}.md` for 6 wired engines (code, content, harvest, forge, gtm, flow) | 4h | — | `rusvel-agent/src/prompts/dept_*.md` (6 new) |
| A3 | **Manifest fields** — add `reasoning_guidance` + `compaction_preserve` to `DepartmentManifest` | 2h | — | `rusvel-core/src/department/manifest.rs` |
| A4 | **Wire framework into dept_chat** — prepend core framework + department guidance to system prompt | 2h | A1, A3 | `rusvel-api/src/department.rs` |
| A5 | **Re-inject after compaction** — re-insert framework as system message after summarization | 2h | A1 | `rusvel-agent/src/lib.rs` |

### Phase B: Parallel Tool Execution (2 days)

| # | Task | Effort | Depends | Files |
|---|------|--------|---------|-------|
| B1 | **`extract_all_tool_calls()`** — replace single-extraction with multi-extraction | 2h | — | `rusvel-agent/src/lib.rs` |
| B2 | **Read-only classification** — use `metadata.read_only` on `ToolDefinition` | 1h | — | `rusvel-agent/src/lib.rs` |
| B3 | **Parallel dispatch** — `JoinSet` for read-only, sequential for stateful | 4h | B1, B2 | `rusvel-agent/src/lib.rs` |
| B4 | **Multi-result messages** — construct multi-part `ToolResult` messages for all results | 3h | B3 | `rusvel-agent/src/lib.rs` |
| B5 | **AG-UI events** — emit `ToolCall`/`ToolResult` for each parallel tool | 2h | B3 | `rusvel-agent/src/lib.rs` |
| B6 | **Tests** — parallel vs sequential classification + execution | 3h | B3 | `rusvel-agent/src/lib.rs` (tests) |

### Phase C: Verification Integration (1.5 days)

| # | Task | Effort | Depends | Files |
|---|------|--------|---------|-------|
| C1 | **Add `verification_chain` to `AgentRuntime`** | 1h | — | `rusvel-agent/src/lib.rs` |
| C2 | **Wire into loop** — after `FinishReason::Stop`, run chain, retry on Fail | 4h | C1 | `rusvel-agent/src/lib.rs` |
| C3 | **Add `max_verification_retries` to `AgentConfig`** | 1h | — | `rusvel-core/src/domain.rs` |
| C4 | **Emit verification events** — `AgentEvent::VerificationStarted/Result` | 2h | C2 | `rusvel-agent/src/lib.rs` |
| C5 | **Default chain wiring** — wire `LlmCritiqueStep` in dept_chat for content/harvest depts | 2h | C1 | `rusvel-api/src/department.rs`, `rusvel-app/src/main.rs` |
| C6 | **Tests** — integration test for verification retry loop | 2h | C2 | `rusvel-agent/src/lib.rs` (tests) |

### Phase D: Graceful Degradation + Permission Mode (1 day)

| # | Task | Effort | Depends | Files |
|---|------|--------|---------|-------|
| D1 | **Graceful degradation** — summary instead of error at iteration limit | 2h | — | `rusvel-agent/src/lib.rs` |
| D2 | **`PermissionMode` enum** — `Default`, `Plan`, `Supervised` | 1h | — | `rusvel-core/src/domain.rs` |
| D3 | **Plan mode** — return tool descriptions without executing | 3h | D2 | `rusvel-agent/src/lib.rs` |
| D4 | **Cancellation token** — `tokio_util::CancellationToken` on `RunState` | 2h | — | `rusvel-agent/src/lib.rs` |
| D5 | **Cancel API endpoint** — `POST /api/dept/{id}/chat/cancel` | 1h | D4 | `rusvel-api/src/department.rs` |

### Phase E: Tool Description Enrichment (1 day)

| # | Task | Effort | Depends | Files |
|---|------|--------|---------|-------|
| E1 | **Audit + enhance remaining tool descriptions** — shell, git, memory, engine tools | 4h | — | `rusvel-builtin-tools/src/shell.rs`, `git.rs`, `memory.rs`; `rusvel-engine-tools/src/` |
| E2 | **Add parameter examples** — concrete example values in JSON Schema `description` fields | 3h | — | All tool registration files |
| E3 | **Error guidance in descriptions** — what errors to expect and how to recover | 1h | — | Priority tools: `edit_file`, `bash`, `grep` |

---

## Sprint Summary

| Phase | Theme | Effort | Parallelizable |
|-------|-------|--------|----------------|
| **A** | Core reasoning framework | 2d | A1+A2 parallel, A3 parallel, then A4+A5 |
| **B** | Parallel tool execution | 2d | B1+B2 parallel, then B3→B4→B5, B6 parallel |
| **C** | Verification integration | 1.5d | C1+C3 parallel, then C2→C4→C5, C6 parallel |
| **D** | Degradation + permissions | 1d | D1 independent, D2→D3, D4→D5 |
| **E** | Tool descriptions | 1d | E1+E2+E3 all parallel |
| **Total** | | **~7.5d** | Phases A+E parallel. B+C sequential. D anytime. |

---

## Acceptance Criteria

| # | Criterion | How to verify |
|---|-----------|---------------|
| 1 | Agent reads files before editing (reasoning framework effect) | Manual: ask agent to "fix bug in X" — should grep/read before writing |
| 2 | Parallel reads complete in ~1 LLM round instead of ~3 | Observe SSE stream — multiple ToolCall events in same iteration |
| 3 | Verification chain catches bad output and retries | Test: `LlmCritiqueStep` returns Fail → agent retries with fix |
| 4 | Iteration limit produces summary, not error | Test: set `max_iterations: 3`, complex task → get summary text |
| 5 | Plan mode describes actions without executing | API: `permission_mode: "plan"` → tool results are descriptions |
| 6 | Cancel stops a running agent | API: start chat, send cancel, verify `Stopped` status |
| 7 | `cargo test -p rusvel-agent` passes | CI |
| 8 | `cargo test -p rusvel-core` passes | CI |

---

## Risk Assessment

| Risk | Mitigation |
|------|-----------|
| Rich system prompts increase token cost | Core framework ~800 tokens; within budget. Monitor via CostTracker. |
| Parallel tool calls cause race conditions | Only `read_only: true` tools run in parallel. Stateful remain sequential. |
| Verification retries add latency | Default `max_verification_retries: 1`. Haiku for critique (cheap + fast). |
| Cancellation token not checked in all paths | Check before each LLM call and before each tool execution. |
