# RUSVEL — Agent Delegation Handbook

> **Purpose:** Copy-paste **instructions, context, rules, and one-line commands** for delegated agents (Claude Code, Cursor, Codex, etc.) so many workers can run in parallel **without** merge chaos.  
> **Canonical sprint list:** [`sprints.md`](./sprints.md) — this doc does not replace task IDs or effort; it **operationalizes** them.

**Last updated:** 2026-03-26

---

## How to use this doc

1. Pick a **task ID** (e.g. `RUSVEL-S2-11`) from the waves below.
2. Paste **GLOBAL RULES (G)** + the task’s **packet** (or the **one-line command**) to the agent.
3. Require the **REPORT TEMPLATE** at the end of every run.
4. **Merge to `main` after each wave** (`cargo test`, then next wave).

---

## GLOBAL RULES (G)

Attach this block to **every** delegated agent:

```text
GLOBAL RULES (RUSVEL)
- Repository: rusvel — Rust workspace + SvelteKit frontend (pnpm, not npm).
- Package managers: cargo (Rust), pnpm (frontend), uv (Python scripts only).
- Architecture: hexagonal — engines depend only on rusvel-core; engines never import adapter crates.
- Scope: touch ONLY files/crates listed in YOUR TASK. Do not refactor unrelated code or “fix” docs unless the task is doc-only.
- Verification: run the exact cargo/pnpm commands in YOUR TASK; fix new warnings you introduced.
- Commit: one logical commit per task, conventional message:
  feat(<scope>): <task-id> <short description>
- If blocked (missing dependency on main), stop and report BLOCKED with what is needed.
```

---

## Dependency table (Sprint 2–5)

| ID | Task | Hard deps | Notes |
|----|------|-----------|--------|
| S2-11 | Context compaction | — | |
| S2-12 | Memory tools | — | |
| S2-13 | Batch LlmPort | Sprint 1 #8 ✅ | |
| S2-14 | Hybrid RAG (RRF) | — | Coordinate file overlap with S2-13 if same PR |
| S2-15 | Frontend manifest | — | **Done** per sprints.md |
| S2-16 | Terminal Web Bridge | Sprint 1 #10 ✅ | |
| S2-17 | Terminal dept tab | S2-16, S2-15 ✅ | Sequential after 16 |
| S3-18 | `delegate_agent` tool | — | |
| S3-19 | `invoke_flow` tool | — | |
| S3-20 | PreToolUse/PostToolUse hooks | ADR-014 ✅ | |
| S3-21 | Tool permissions | ADR-014 ✅ | |
| S3-22 | Event triggers | S3-18, S3-19 | |
| S3-23 | Self-correction loops | Sprint 1 #8 ✅ | Hooks (S3-20) help |
| S3-24 | Terminal agent visibility | S2-16, S3-18 | |
| S4-25 | Durable FlowEngine | — | |
| S4-26 | Playbooks UI | S3-18, S3-19, S3-22 | |
| S4-27 | AG-UI / SSE mapping | — | |
| S4-28 | Terminal flow/playbook panes | S2-16, S4-25, S4-26 | |
| S5-29 | Executive brief | S3-18 | |
| S5-30 | Starter kits | ADR-014 ✅ | |
| S5-31 | Self-improving KB | S2-14 | |
| S5-32 | Streamable HTTP MCP | — | |
| S5-33 | Terminal CDP + TUI | S2-16 | See `cdp-browser-bridge.md` |

---

## Execution waves (priority + parallelism)

| Wave | Tasks (parallel where listed) | Merge before next wave |
|------|--------------------------------|-------------------------|
| **S2-α** | S2-11 ∥ S2-12 | ✅ |
| **S2-β** | S2-13 ∥ S2-14 | ✅ (split crates to avoid two agents editing one file) |
| **S2-γ** | S2-16 → S2-17 | ✅ |
| **S3-α** | S3-18 ∥ S3-19 ∥ S3-20 ∥ S3-21 ∥ S3-23 | ✅ |
| **S3-β** | S3-22 | ✅ |
| **S3-γ** | S3-24 | ✅ |
| **S4-α** | S4-25 ∥ S4-27 | ✅ |
| **S4-β** | S4-26 | ✅ |
| **S4-γ** | S4-28 | ✅ |
| **S5** | S5-29, S5-30, S5-31, S5-32 per deps; S5-33 last | ✅ |

**Rule:** Prefer **one agent per crate** in a wave. If two tasks touch `rusvel-api`, run them **sequentially** or **split by module/file** in the packet.

---

## REPORT TEMPLATE (required from every agent)

```markdown
## Task ID:
## Branch / commit:
## Files changed:
## What I implemented:
## Tests run (commands + pass/fail):
## Known limitations / follow-ups:
## Blocked: yes/no — if yes, on what:
```

---

## Task packets + one-line commands

### Sprint 2

#### RUSVEL-S2-11 — Context compaction

| Field | Content |
|-------|---------|
| **Context** | Long chats exceed useful context; summarize older turns, keep a recent tail. |
| **Scope** | `crates/rusvel-agent/` only (+ its tests). |
| **Do** | When history exceeds threshold (e.g. 30 messages), collapse oldest block into one summary message; keep N recent messages; use existing LlmPort/AgentPort patterns and cheap tier where applicable. |
| **Verify** | `cargo test -p rusvel-agent` |

**One-line command:**

```text
Read GLOBAL RULES (G). Task RUSVEL-S2-11 Context Compaction: only crates/rusvel-agent/. Implement history compaction (threshold + summary + recent tail); add tests; cargo test -p rusvel-agent; commit; report with REPORT TEMPLATE.
```

---

#### RUSVEL-S2-12 — Memory tools

| Field | Content |
|-------|---------|
| **Context** | Agents need first-class memory ops as tools backed by MemoryPort. |
| **Scope** | `crates/rusvel-builtin-tools/`; `crates/rusvel-memory/` only if wiring requires it. |
| **Do** | Expose `memory_read`, `memory_write`, `memory_search`, `memory_delete` (or names matching project conventions); register in `register_all()`; `searchable` where appropriate. |
| **Verify** | `cargo test -p rusvel-builtin-tools -p rusvel-memory` |

**One-line command:**

```text
Read GLOBAL RULES (G). Task RUSVEL-S2-12 Memory Tools: wire MemoryPort in rusvel-builtin-tools; register tools; tests; cargo test -p rusvel-builtin-tools -p rusvel-memory; commit; report.
```

---

#### RUSVEL-S2-13 — Batch API

| Field | Content |
|-------|---------|
| **Context** | Async batch jobs should use cheaper batch pricing where the provider supports it. |
| **Scope** | `crates/rusvel-core/` (LlmPort), `crates/rusvel-llm/`; callers only to satisfy compilation. |
| **Do** | Add `submit_batch` / `poll_batch` (or equivalent) with clear semantics; implement for primary providers as feasible. |
| **Verify** | `cargo test -p rusvel-core -p rusvel-llm` |

**One-line command:**

```text
Read GLOBAL RULES (G). Task RUSVEL-S2-13 Batch LlmPort: batch submit/poll on LlmPort in rusvel-core + rusvel-llm; minimal callers; tests; cargo test -p rusvel-core -p rusvel-llm; commit; report.
```

---

#### RUSVEL-S2-14 — Hybrid RAG

| Field | Content |
|-------|---------|
| **Context** | Combine FTS5 and vector (LanceDB) results for better retrieval. |
| **Scope** | `rusvel-core` (RRF/fusion), `rusvel-api` knowledge routes, `rusvel-vector` / `rusvel-memory` as needed. |
| **Do** | Reciprocal Rank Fusion (or agreed fusion); hybrid mode on knowledge search API; tests. |
| **Verify** | `cargo test -p rusvel-core -p rusvel-api -p rusvel-vector` (adjust if crate split differs) |

**One-line command:**

```text
Read GLOBAL RULES (G). Task RUSVEL-S2-14 Hybrid RAG: RRF fusion FTS5 + LanceDB; hybrid search API; tests in touched crates; commit; report.
```

---

#### RUSVEL-S2-16 — Terminal Web Bridge

| Field | Content |
|-------|---------|
| **Context** | Browser UI for PTY sessions (WebSocket + xterm.js). |
| **Scope** | `crates/rusvel-api/`, `frontend/` (`/terminal`, xterm). |
| **Do** | WebSocket route, session binding, `/terminal` page with xterm.js; align with `native-terminal-multiplexer.md` / TerminalPort. |
| **Verify** | `cargo test -p rusvel-api`; `cd frontend && pnpm check` (and `pnpm build` if routes/components added) |

**One-line command:**

```text
Read GLOBAL RULES (G). Task RUSVEL-S2-16 Terminal Web: WebSocket + xterm.js + /terminal in rusvel-api + frontend; pnpm check; cargo test -p rusvel-api; commit; report.
```

---

#### RUSVEL-S2-17 — Terminal dept integration

| Field | Content |
|-------|---------|
| **Context** | Terminal accessible from each department panel. |
| **Scope** | `frontend/` dept shell (tabs/panes). |
| **Do** | “Terminal” tab; lazy-load; integrate with S2-16 route or embed. |
| **Depends** | S2-16 merged |
| **Verify** | `pnpm check` in `frontend/` |

**One-line command:**

```text
Read GLOBAL RULES (G). Task RUSVEL-S2-17 Dept Terminal Tab: requires S2-16 on main; add Terminal tab to dept panel; lazy panes; pnpm check; commit; report.
```

---

### Sprint 3

#### RUSVEL-S3-18 — `delegate_agent` tool

| **Scope** | `rusvel-builtin-tools`, `rusvel-agent` if loop hooks needed |
| **Do** | Sub-agent with persona, tool allowlist, token/time budget, max depth; safe defaults |
| **Verify** | `cargo test -p rusvel-builtin-tools -p rusvel-agent` |

```text
Read GLOBAL RULES (G). Task RUSVEL-S3-18 delegate_agent: builtin-tools + minimal agent wiring; budget caps; tests; cargo test -p rusvel-builtin-tools -p rusvel-agent; commit; report.
```

---

#### RUSVEL-S3-19 — `invoke_flow` tool

| **Scope** | `rusvel-builtin-tools`, `flow-engine` / API glue |
| **Verify** | `cargo test -p rusvel-builtin-tools -p flow-engine` (and API if wired) |

```text
Read GLOBAL RULES (G). Task RUSVEL-S3-19 invoke_flow: agents start FlowEngine DAGs and receive results; tests; commit; report.
```

---

#### RUSVEL-S3-20 — PreToolUse / PostToolUse hooks

| **Scope** | `rusvel-core` (types), `rusvel-agent` (loop) |
| **Verify** | `cargo test -p rusvel-core -p rusvel-agent` |

```text
Read GLOBAL RULES (G). Task RUSVEL-S3-20 Tool hooks: HookPoint + ToolHook trait + agent loop; tests; commit; report.
```

---

#### RUSVEL-S3-21 — Tool permissions

| **Scope** | `rusvel-core`, `rusvel-tool`, `rusvel-app` if composition needed |
| **Verify** | `cargo test -p rusvel-core -p rusvel-tool` |

```text
Read GLOBAL RULES (G). Task RUSVEL-S3-21 Tool permissions: per-dept ToolPermissionMode; enforce on registry/exec path; tests; commit; report.
```

---

#### RUSVEL-S3-22 — Event triggers

| **Depends** | S3-18, S3-19 |
| **Scope** | `rusvel-event`, jobs/app wiring |
| **Verify** | scoped crate tests |

```text
Read GLOBAL RULES (G). Task RUSVEL-S3-22 Event triggers: pattern subscriptions start agent/flow; requires S3-18+19 merged; tests; commit; report.
```

---

#### RUSVEL-S3-23 — Self-correction loops

| **Scope** | `rusvel-agent` |
| **Verify** | `cargo test -p rusvel-agent` |

```text
Read GLOBAL RULES (G). Task RUSVEL-S3-23 Self-correction: VerificationStep + tests; cargo test -p rusvel-agent; commit; report.
```

---

#### RUSVEL-S3-24 — Terminal agent visibility

| **Depends** | S2-16, S3-18 |
| **Scope** | `frontend/`, small API additions if needed |
| **Verify** | `pnpm check`, API tests if added |

```text
Read GLOBAL RULES (G). Task RUSVEL-S3-24 Terminal agent visibility: requires S2-16+S3-18; delegation UI + terminal_open/watch tools; commit; report.
```

---

### Sprint 4

| ID | One-line command |
|----|------------------|
| S4-25 | `Read GLOBAL RULES (G). Task RUSVEL-S4-25 Durable FlowEngine: checkpoint/resume/retry per node; tests; commit; report.` |
| S4-26 | `Read GLOBAL RULES (G). Task RUSVEL-S4-26 Playbooks: /playbooks + JSON pipelines; deps S3-18/19/22; commit; report.` |
| S4-27 | `Read GLOBAL RULES (G). Task RUSVEL-S4-27 AG-UI mapping: SSE → AG-UI schema; tests; commit; report.` |
| S4-28 | `Read GLOBAL RULES (G). Task RUSVEL-S4-28 Terminal flow/playbook panes: deps S2-16+S4-25+S4-26; commit; report.` |

---

### Sprint 5

| ID | One-line command |
|----|------------------|
| S5-29 | `Read GLOBAL RULES (G). Task RUSVEL-S5-29 Executive brief: forge mission today + delegate; deps S3-18; commit; report.` |
| S5-30 | `Read GLOBAL RULES (G). Task RUSVEL-S5-30 Starter kits: dept bundles via capability/!build; commit; report.` |
| S5-31 | `Read GLOBAL RULES (G). Task RUSVEL-S5-31 Self-improving KB: auto-index outputs; deps S2-14; commit; report.` |
| S5-32 | `Read GLOBAL RULES (G). Task RUSVEL-S5-32 Streamable HTTP MCP: Axum + sessions + stdio fallback; commit; report.` |
| S5-33 | `Read GLOBAL RULES (G). Task RUSVEL-S5-33 Terminal CDP+TUI: deps S2-16 + cdp-browser-bridge plan; commit; report.` |

---

## Worktree / parallel agent tips

| Tip | Why |
|-----|-----|
| One branch per task ID | Clean review and revert |
| Subagents need Edit + `cargo` + `pnpm` allowlisted | Avoid “plan only” failures |
| Merge wave before spawning next wave | Reduces conflict surface |
| Do not assign two agents the same file | Obvious but frequent failure mode |

---

## Related docs

- [`sprints.md`](./sprints.md) — task status, effort, shipped commits
- [`agent-orchestration.md`](./agent-orchestration.md) — delegate_agent, messaging
- [`native-terminal-multiplexer.md`](./native-terminal-multiplexer.md) — terminal phases
- [`agent-sdk-features.md`](./agent-sdk-features.md) — SDK-aligned features
- [`../design/architecture-v2.md`](../design/architecture-v2.md) — ports and boundaries
