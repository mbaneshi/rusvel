# Sprint: Agent Intelligence Layer

> **Theme:** Make agents smart, not just functional
> **Date:** 2026-03-30 | **Status:** Planned
> **ADR:** [ADR-015](../design/adr-015-agent-intelligence-layer.md)
> **Prerequisite:** None (all changes within `rusvel-agent` + `rusvel-engine-tools`)
> **Estimated effort:** 5-7 working days across 3 phases

---

## Phase A: Quick Wins (1-2 days)

> Text changes + small code additions. No new structs. Ship in one session.

| # | Task | Crate | Effort | Depends |
|---|------|-------|--------|---------|
| A1 | **Enrich engine tool descriptions** — expand 12 engine tools from 1-sentence to structured format (WHEN TO USE / PREREQUISITES / WORKFLOW / TIPS) | `rusvel-engine-tools` | 2h | — |
| A2 | **Per-failure reflection** — inject "why did this fail?" system message after EVERY tool error (not just after 3) | `rusvel-agent` | 1h | — |
| A3 | **5-failure bailout** — at 5 consecutive failures, tell agent to stop and return summary to user | `rusvel-agent` | 30m | A2 |
| A4 | **Loop detection** — track `(tool_name, args_hash)` and warn on repeated identical calls | `rusvel-agent` | 1h | — |
| A5 | **Progress checkpoints** — inject checkpoint system message every 10 iterations | `rusvel-agent` | 30m | — |

**Acceptance criteria:**
- `cargo test -p rusvel-agent` passes
- `cargo test -p rusvel-engine-tools` passes (if tests exist)
- Manual chat test: give agent a multi-step task, observe reflection on error and checkpoint messages in SSE stream

---

## Phase B: Prompt Templates (2-3 days)

> New `PromptTemplate` struct + expanded persona definitions.

| # | Task | Crate | Effort | Depends |
|---|------|-------|--------|---------|
| B1 | **`PromptSection` + `PromptTemplate` structs** — new file `rusvel-agent/src/prompt_template.rs` with `render()`, `trim_to_budget()` | `rusvel-agent` | 3h | — |
| B2 | **Universal sections** — define Reasoning Framework, Tool Selection Guide, Error Recovery Rules as constants | `rusvel-agent` | 2h | B1 |
| B3 | **Expand 10 personas** — each gets ~200 words of instructions + output format rules | `rusvel-agent` | 3h | B1 |
| B4 | **Department knowledge injection** — read `DepartmentManifest` description/actions into prompt section at chat time | `rusvel-agent`, `rusvel-api` | 2h | B1 |
| B5 | **Tool selection guide generation** — auto-generate from available `ToolDefinition`s (name + 1-line description + when-to-use) | `rusvel-agent` | 2h | B1 |
| B6 | **Wire `PromptTemplate` into agent loops** — replace `config.instructions` assembly in both `run_streaming_loop` and `run` | `rusvel-agent` | 2h | B1-B5 |
| B7 | **Tests** — unit tests for `PromptTemplate::render()`, `trim_to_budget()`, and persona assembly | `rusvel-agent` | 1h | B6 |

**Acceptance criteria:**
- `PromptTemplate::render()` produces a string with all sections separated by `---`
- `trim_to_budget(token_limit)` drops lowest-priority sections first
- `AgentConfig.instructions` still overrides template (backwards compat)
- All existing `rusvel-agent` tests pass
- System prompt visible in SSE debug output (first message of chat)

---

## Phase C: Smart Compaction (2 days)

> Rewrite `compact_messages()` for better context preservation.

| # | Task | Crate | Effort | Depends |
|---|------|-------|--------|---------|
| C1 | **Token-based threshold** — replace `messages.len() > 30` with `estimate_tokens(messages) > 80_000` | `rusvel-agent` | 1h | — |
| C2 | **Structured summary prompt** — replace generic "summarize" with template requesting Goal / Decisions / Files / Tool Results / Current State sections | `rusvel-agent` | 2h | — |
| C3 | **Message pinning** — add `pinned` flag check in `compact_messages()`; pinned messages skip summarization | `rusvel-agent` | 2h | — |
| C4 | **Auto-pin heuristics** — pin messages containing file write confirmations, user approvals, or error recovery decisions | `rusvel-agent` | 2h | C3 |
| C5 | **Tests** — test compaction with pinned messages, token-based triggering, structured output | `rusvel-agent` | 1h | C1-C4 |

**Acceptance criteria:**
- Compaction triggers based on estimated tokens, not message count
- Pinned messages survive compaction verbatim
- Structured summary contains named sections (Goal, Decisions, Files, etc.)
- Long conversation (60+ messages) retains key file paths and decisions after compaction

---

## Files Changed

```
crates/rusvel-agent/src/
├── lib.rs                  # A2-A5 (error recovery, checkpoints, loop detect)
│                           # B6 (wire PromptTemplate)
│                           # C1-C4 (compact_messages rewrite)
├── prompt_template.rs      # B1 NEW — PromptTemplate + PromptSection
├── persona.rs              # B3 (expand 10 personas)
└── context_pack.rs         # unchanged (already outputs markdown)

crates/rusvel-engine-tools/src/
├── code.rs                 # A1 (enrich descriptions)
├── content.rs              # A1 (enrich descriptions)
└── harvest.rs              # A1 (enrich descriptions)

crates/rusvel-core/src/
└── domain.rs               # no changes needed (metadata already supports pinning)

crates/rusvel-api/src/
└── chat.rs                 # B4 (pass DepartmentManifest to prompt builder)
```

---

## Risks & Mitigations

| Risk | Mitigation |
|------|-----------|
| Longer system prompts increase cost | Token budget trimming in `PromptTemplate`; measure cost delta in dev |
| Reflection prompts may confuse weaker models (Haiku) | Keep reflection prompts short and direct; test with Haiku tier |
| Token estimation is approximate | Use `len() / 4` as conservative estimate; actual tokenizer is overkill for this |
| Pinned messages could accumulate and defeat compaction | Cap at 10 pinned messages; oldest pins dropped first |
| Backwards compat — existing `AgentConfig.instructions` users | `instructions` overrides template entirely; no breaking change |

---

## Success Metrics

After Phase A (measurable in manual testing):
- Agent recovers from tool errors without repeating the same call
- Multi-step tasks complete in fewer iterations (checkpoint self-assessment)
- Engine tools are used correctly (no more `code_search` without `code_analyze`)

After Phase B (measurable in chat quality):
- System prompts are 1000-1500 tokens (vs current ~50)
- Agent explains reasoning before acting
- Tool selection matches the task (no `bash cat file.rs` when `read_file` exists)

After Phase C (measurable in long conversations):
- Context survives 60+ messages without losing key file paths
- Compaction triggers earlier for verbose tool outputs, later for short conversations
- Pinned decisions survive across multiple compaction cycles
