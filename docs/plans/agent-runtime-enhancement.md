# Agent Runtime Enhancement Plan

> **Goal:** Close the quality gap between RUSVEL agents and production-grade agent runtimes (Claude Code, Cursor, etc.)
> **Status:** Design — no implementation yet
> **Date:** 2026-03-30

---

## Problem Statement

RUSVEL agents have the architecture (tool loop, streaming, hooks, verification) but lack the **intelligence layer** that makes agents actually effective. The 5 gaps below explain why agents feel "dumb" despite having the right plumbing.

## Current State (as of 2026-03-30)

| Area | Current | Where |
|------|---------|-------|
| System prompts | 6-10 words per persona (e.g. "Write clean, idiomatic code. Follow best practices.") | `rusvel-agent/src/persona.rs:93-157` |
| Tool descriptions | Built-in tools: decent (multi-line with tips). Engine tools: 1 sentence each | `rusvel-builtin-tools/src/`, `rusvel-engine-tools/src/` |
| Max iterations | 50 (configurable via `AgentConfig.max_iterations`) | `rusvel-agent/src/lib.rs:45` |
| Error recovery | Basic: 3 consecutive failures triggers reflection prompt | `rusvel-agent/src/lib.rs:823-838` |
| Context management | Compact at 30 messages, keep 10 recent, LLM-summarize rest | `rusvel-agent/src/lib.rs:524-609` |
| Verification | `VerificationChain` with LLM critique + regex rules (exists but rarely used) | `rusvel-agent/src/verification.rs` |

## The 5 Enhancements

---

### 1. Rich System Prompt Templates

**Gap:** Personas have ~10 words of instruction. The LLM gets no reasoning framework, no tool selection guidance, no output format rules. It's flying blind.

**Design:**

Replace flat `instructions: String` with a **composable prompt template** system. Each prompt is assembled from sections:

```
┌─────────────────────────────────────┐
│ 1. Identity & Role                  │  ← from persona (expanded)
│ 2. Reasoning Framework              │  ← NEW: universal
│ 3. Tool Selection Guide             │  ← NEW: per-tool-set
│ 4. Error Recovery Rules             │  ← NEW: universal
│ 5. Output Format Rules              │  ← NEW: per-persona
│ 6. Department Knowledge             │  ← NEW: per-department
│ 7. Context Pack                     │  ← existing: goals, events
│ 8. Rules                            │  ← existing: user-defined
└─────────────────────────────────────┘
```

**Section details:**

**1. Identity & Role** (expanded from current 10 words → ~200 words per persona):
```
You are a Software Engineer working in the RUSVEL virtual agency.

Your expertise: writing clean, idiomatic Rust and TypeScript code.
You follow hexagonal architecture principles — engines depend only on port traits,
never on adapter implementations.

When given a task:
- Read existing code before modifying it
- Prefer editing existing files over creating new ones
- Keep functions small and single-purpose
- Don't add features beyond what was asked
```

**2. Reasoning Framework** (universal, ~150 words):
```
Before acting, briefly state your plan (1-3 sentences).
After each tool result, assess: did this move me forward?
If stuck after 2 attempts, step back and try a different approach.
Before returning your final answer, verify:
  - Did I complete everything that was asked?
  - Did I introduce any issues?
  - Is my response concise and actionable?
```

**3. Tool Selection Guide** (generated from available tools, ~200 words):
```
Tool selection rules:
- To read a file you know exists → read_file
- To find files by name/pattern → glob
- To search file contents → grep
- To make small edits → edit_file (NOT write_file)
- To create new files → write_file
- To run commands → bash
- Do NOT use bash for: reading files (use read_file), searching (use grep/glob)
- Do NOT use write_file for small edits (use edit_file)
- If you need a tool not listed here → call tool_search first
```

**4. Error Recovery Rules** (universal, ~100 words):
```
When a tool fails:
1. Read the error message carefully
2. Explain what went wrong (1 sentence)
3. Try a different approach — don't repeat the same call
If the same tool fails twice: try a completely different strategy
If 3 tools fail in a row: stop, summarize what you've tried, and ask for guidance
```

**5. Output Format Rules** (per-persona):
- CodeWriter: "Show code changes, not explanations. Use diffs when possible."
- Reviewer: "List issues as bullet points with severity. Include file:line references."
- Researcher: "Summarize findings. Cite sources. Separate facts from opinions."

**6. Department Knowledge** (per-department, loaded from `DepartmentManifest`):
```
You are operating in the {dept_name} department.
Available actions: {manifest.actions}
Domain context: {manifest.description}
Related departments: {manifest.dependencies}
```

**Implementation approach:**

- New struct `PromptTemplate` in `rusvel-agent` with `sections: Vec<PromptSection>`
- Each `PromptSection` has `name`, `content`, `priority` (for token budget trimming)
- `PromptTemplate::render()` concatenates sections with separators
- `PersonaCatalog` stores `PromptTemplate` per persona instead of flat `instructions`
- Universal sections (reasoning, error recovery) defined once, shared across personas
- Department knowledge injected at runtime via `DepartmentManifest`

**Files to change:**
- `rusvel-agent/src/persona.rs` — expanded persona definitions + template construction
- `rusvel-agent/src/lib.rs` — use `PromptTemplate::render()` instead of raw `config.instructions`
- New file: `rusvel-agent/src/prompt_template.rs` — `PromptTemplate` + `PromptSection` structs

---

### 2. Enhanced Engine Tool Descriptions

**Gap:** Engine tools have 1-sentence descriptions. The LLM doesn't know when to use them, what they return, or what can go wrong.

**Current engine tool descriptions:**
```
code_analyze    → "Analyze a repository: parse Rust files, build symbol graph, compute metrics, and index for search."
code_search     → "Search previously indexed code symbols using BM25. Requires analyze() to have been called first."
content_draft   → "Draft new AI-generated content on a given topic."
content_adapt   → "Adapt existing content for a target platform."
content_publish → "Publish approved content to a platform. Content must be approved first."
content_list    → "List content items for a session, optionally filtered by status."
content_approve → "Mark a content item as human-approved (required before publishing)."
harvest_scan    → "Scan for freelance opportunities using the mock source. Returns discovered opportunities."
harvest_score   → "Re-score an existing opportunity and update its stored score."
harvest_propose → "Generate a tailored proposal for a stored opportunity."
harvest_list    → "List opportunities, optionally filtered by pipeline stage."
harvest_pipeline→ "Get pipeline statistics for a session (total count, breakdown by stage)."
```

**Target format** (example for `content_draft`):
```
Draft new AI-generated content on a given topic.

WHEN TO USE: Creating blog posts, social media content, newsletters, or technical articles.
WHEN NOT TO USE: Editing existing content (use content_adapt), publishing (use content_publish).

PREREQUISITES: None — this creates a new draft from scratch.

PARAMETERS:
- topic (required): The subject to write about
- platform (optional): Target platform affects tone/length (linkedin, twitter, devto, blog)
- style (optional): Writing style (technical, conversational, promotional)

RETURNS: Content item with id, title, body, status="draft"

WORKFLOW: content_draft → content_approve → content_publish
The draft must be approved before it can be published.

TIPS:
- Be specific with the topic for better results
- Set platform early — adapting later costs an extra LLM call
```

**Implementation approach:**
- Update descriptions directly in `rusvel-engine-tools/src/{code,content,harvest}.rs`
- Follow the same pattern already used in built-in tools (WHEN TO USE / TIPS / etc.)
- Add WORKFLOW sections showing tool chains (draft→approve→publish)
- Add PREREQUISITES where relevant (code_search requires code_analyze first)

**Files to change:**
- `rusvel-engine-tools/src/code.rs`
- `rusvel-engine-tools/src/content.rs`
- `rusvel-engine-tools/src/harvest.rs`

---

### 3. Progress Checkpoints (Mid-Run Reflection)

**Gap:** Even with 50 iterations, agents can spin in circles without realizing they're stuck. No mid-run self-assessment.

**Design:**

Insert **checkpoint prompts** at iteration 10, 20, 30, 40:

```rust
if iteration > 0 && iteration % 10 == 0 {
    messages.push(LlmMessage {
        role: LlmRole::System,
        content: Content::text(format!(
            "[CHECKPOINT — iteration {iteration}/{max_iter}] \
             Briefly assess: Are you making progress toward the goal? \
             If stuck, change strategy. If done, return your answer."
        )),
    });
}
```

This is lightweight — just a system message injection, no new structs.

**Additionally:** Track tool call patterns to detect loops:
```rust
// If the same tool was called with same args 2+ times, inject warning
if recent_calls.contains(&(tool_name, tool_args_hash)) {
    messages.push(system_msg("[WARNING] You called the same tool with the same arguments again. This suggests you're in a loop. Try a different approach."));
}
```

**Files to change:**
- `rusvel-agent/src/lib.rs` — checkpoint injection in both sync and streaming loops
- Track `recent_calls: Vec<(String, u64)>` (tool name + args hash)

---

### 4. Structured Error Recovery

**Gap:** Current recovery is minimal — only triggers after 3 consecutive failures. No per-failure reflection, no strategy tracking.

**Current state** (already partially implemented at `lib.rs:823-838`):
```rust
if is_error {
    consecutive_failures += 1;
    if consecutive_failures >= 3 {
        // reflection prompt injected
    }
} else {
    consecutive_failures = 0;
}
```

**Enhanced design:**

```rust
if is_error {
    consecutive_failures += 1;

    // Level 1: After EVERY failure — force the LLM to reflect
    messages.push(system_msg(format!(
        "[TOOL FAILED] `{tool_name}` returned error: {error_text}\n\
         Before your next action, briefly state:\n\
         1. Why this failed\n\
         2. What you'll try differently"
    )));

    // Level 2: After 3 failures — force strategy change
    if consecutive_failures >= 3 {
        messages.push(system_msg(
            "[STRATEGY RESET] 3 consecutive failures. \
             Your current approach isn't working. \
             List 2-3 alternative strategies and pick the most promising one."
        ));
    }

    // Level 3: After 5 failures — suggest asking for help
    if consecutive_failures >= 5 {
        messages.push(system_msg(
            "[STUCK] 5 consecutive failures. \
             Summarize what you've tried and what's blocking you. \
             Return this summary to the user — don't keep trying."
        ));
    }
} else {
    consecutive_failures = 0;
}
```

**Pre-tool validation** (new, lightweight):
```rust
// Before calling a destructive tool, inject a confirmation prompt
if tool_metadata.get("destructive") == Some(&json!(true)) {
    messages.push(system_msg(format!(
        "[CAUTION] `{tool_name}` is destructive. \
         Verify this is the right action before proceeding."
    )));
}
```

**Files to change:**
- `rusvel-agent/src/lib.rs` — enhance the existing error recovery block in both loops

---

### 5. Smarter Context Compaction

**Gap:** At 30 messages, everything except the last 10 gets lossy-summarized. Critical tool results, decisions, and file paths get lost.

**Design — 3 improvements:**

#### 5a. Token-based threshold (not message count)

```rust
// Instead of:  if messages.len() > 30
// Use:         if estimated_tokens(messages) > TOKEN_BUDGET
const TOKEN_BUDGET: usize = 80_000;  // ~60% of context window, leave room for response

fn estimate_tokens(messages: &[LlmMessage]) -> usize {
    messages.iter().map(|m| content_to_plain(&m.content).len() / 4).sum()
}
```

This prevents premature compaction of short messages while catching conversations with very long tool outputs earlier.

#### 5b. Structured summarization prompt

Replace the current generic "summarize the conversation" with:

```
Summarize the prior conversation into a structured context block.
Preserve these categories:

## Goal
What is the user trying to accomplish?

## Decisions Made
Key choices and their rationale.

## Files Touched
File paths that were read, created, or modified.

## Tool Results That Matter
Important outputs that inform next steps (not routine reads).

## Current State
Where we left off. What's the next step?

Output in this exact markdown format. Be concise but preserve specifics
(file paths, function names, error messages).
```

#### 5c. Message pinning

Allow tool results to be "pinned" so they survive compaction:

```rust
// In LlmMessage metadata:
{ "pinned": true }

// In compact_messages: skip pinned messages from summarization
let to_summarize: Vec<_> = messages[system_len..suffix_start]
    .iter()
    .filter(|m| !is_pinned(m))
    .collect();
```

Agents can pin messages programmatically (e.g. "this file listing is important"), and the verification chain can pin messages containing key decisions.

**Files to change:**
- `rusvel-agent/src/lib.rs` — `compact_messages()` rewrite
- `rusvel-core/src/domain.rs` — add `pinned` support to message metadata

---

## Marketplace Connection

These enhancements are prerequisites for the capability marketplace (see `docs/plans/capability-marketplace-design.md`). The marketplace will share:

| Marketplace Item | Requires Enhancement |
|-----------------|---------------------|
| Prompt templates ("Expert Code Reviewer") | #1 — Rich system prompts |
| Tool bundles with schemas | #2 — Enhanced tool descriptions |
| Workflow recipes (multi-step) | #3 — Progress checkpoints (agents need to handle 20+ steps) |
| Reflection rules ("retry on auth failure") | #4 — Error recovery framework |
| Long-running agent patterns | #5 — Context compaction |

Without these runtime improvements, marketplace items won't work well — sharing a skill doesn't help if the agent can't follow complex instructions.

---

## Implementation Order

| Phase | Enhancement | Effort | Files Changed |
|-------|------------|--------|---------------|
| **A** | #2 Engine tool descriptions | S (text only) | 3 files in `rusvel-engine-tools` |
| **A** | #3 Progress checkpoints | S (few lines) | `rusvel-agent/src/lib.rs` |
| **A** | #4 Structured error recovery | S (expand existing) | `rusvel-agent/src/lib.rs` |
| **B** | #1 Rich system prompts | M (new struct + 10 personas) | 3 files in `rusvel-agent` |
| **C** | #5 Smarter context compaction | M-L (token counting, pinning) | 2 files |

**Phase A** is pure text changes and small code additions — can ship in one session.
**Phase B** requires the new `PromptTemplate` struct but no architectural changes.
**Phase C** touches the compaction algorithm and message metadata — needs careful testing.

---

## Non-Goals

- **Changing the agent loop architecture** — the current tool-use loop is fine
- **Adding sub-agent delegation** — covered in `docs/plans/agent-orchestration.md`
- **Changing streaming/AG-UI protocol** — the wire format is good
- **Replacing the verification chain** — it works, just needs to be used more
