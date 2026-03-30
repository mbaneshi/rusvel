# Why Claude Code Works But Your App Doesn't
## A Complete Technical Proposal for RUSVEL Agent Intelligence

> **The Solo Builder's AI-Powered Virtual Agency — Making the AI Actually Intelligent**
>
> Date: 2026-03-30 | Author: RUSVEL Engineering

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [The Gap: A Side-by-Side Dissection](#2-the-gap-a-side-by-side-dissection)
3. [Deep Dive: The Agent Loop](#3-deep-dive-the-agent-loop)
4. [Deep Dive: System Prompts](#4-deep-dive-system-prompts)
5. [Deep Dive: Tool Descriptions](#5-deep-dive-tool-descriptions)
6. [Deep Dive: Context Management](#6-deep-dive-context-management)
7. [Deep Dive: Error Recovery & Reflection](#7-deep-dive-error-recovery--reflection)
8. [Deep Dive: Multi-Agent Orchestration](#8-deep-dive-multi-agent-orchestration)
9. [Deep Dive: Human-in-the-Loop](#9-deep-dive-human-in-the-loop)
10. [Deep Dive: Streaming & Feedback Loops](#10-deep-dive-streaming--feedback-loops)
11. [The Marketplace Connection](#11-the-marketplace-connection)
12. [Implementation Roadmap](#12-implementation-roadmap)
13. [File Impact Map](#13-file-impact-map)
14. [Appendix: Full Code Audit](#14-appendix-full-code-audit)

---

## 1. Executive Summary

RUSVEL's agent runtime has the right architecture — hexagonal ports, tool-use loop, streaming events, deferred tool loading, department scoping. But the *intelligence layer* is thin. The difference between Claude Code (which feels magical) and RUSVEL's agents (which feel mechanical) comes down to **six missing layers of sophistication**:

| Layer | Claude Code | RUSVEL Today | Impact |
|-------|-------------|--------------|--------|
| **System prompts** | 5000+ words with reasoning frameworks, tool guidance, error strategies | ~200 words generic persona | Agent doesn't know *how* to think |
| **Tool descriptions** | Multi-paragraph with examples, constraints, "when NOT to use" | 1-2 sentence summaries | Agent misuses or ignores tools |
| **Loop depth** | Unlimited turns, parallel tool calls | Max 10 iterations, serial only | Complex tasks fail mid-way |
| **Error recovery** | Structured reflection, retry with context, validation | Forward-only, error-as-text | Errors cascade into nonsense |
| **Context persistence** | Full window + CLAUDE.md re-injection + prompt caching | 50 messages, lossy summarization at 30 | Agent forgets what it's doing |
| **Feedback loops** | Interactive, interruptible, permission gates | One-way streaming, no interruption | Can't course-correct |

**This is not a rewrite.** The architecture is sound. This is about filling in the intelligence that makes the architecture *work*. Every recommendation below operates within the existing hexagonal design — `rusvel-agent` (adapter), `rusvel-core` (domain), `rusvel-builtin-tools` (adapter).

---

## 2. The Gap: A Side-by-Side Dissection

### 2.1 What Happens When You Ask Claude Code to "Add a login page"

```
Turn 1: Claude Code reads the system prompt (5000+ words)
        → Knows it should search before writing
        → Knows to check existing auth patterns
        → Knows to use Read tool (not cat/head/tail)
        → Knows to verify its work after writing

Turn 2: Searches for existing auth code (Grep tool)
        → Tool description told it to use regex patterns
        → Finds existing auth middleware

Turn 3: Reads the auth middleware (Read tool)
        → Tool description told it to read before editing
        → Understands the existing pattern

Turn 4: Reads the router config (Read tool, parallel with Turn 3)
        → Multiple reads happen concurrently
        → Understands where to add the route

Turn 5: Writes the login component (Write tool)
        → System prompt told it to match existing style
        → Uses patterns found in Turn 2-4

Turn 6: Updates the router (Edit tool)
        → System prompt told it to prefer Edit over Write for existing files

Turn 7: Runs tests (Bash tool)
        → System prompt told it to verify after writing
        → If tests fail → reads error, fixes, retries

Turn 8: Responds with summary
        → System prompt told it to be concise
        → Shows what was created and why
```

**8 turns, 5-6 tool calls, self-verified result.**

### 2.2 What Happens When RUSVEL's Agent Gets the Same Request

```
Turn 1: Agent reads system prompt (~200 words)
        "You are the RUSVEL AI assistant for Alex, a Solo Founder."
        → No guidance on search-before-write
        → No guidance on which tools to use when
        → No reasoning framework

Turn 2: Agent writes code directly (write_file tool)
        → Didn't search for existing patterns
        → Didn't read the router
        → Guesses at the project structure

Turn 3: Maybe reads an error if the code was wrong
        → Tool description didn't tell it what to do on failure
        → No reflection step

Turn 4-10: Flails or produces something that doesn't integrate
        → Hits MAX_ITERATIONS = 10 and stops
        → "I've completed the task" (but it didn't)
```

**The same model (Claude Sonnet), dramatically different results.** The difference is entirely in the harness.

### 2.3 The Compounding Effect

Each gap multiplies the others:

```
Thin system prompt
  → Agent doesn't know to search first
    → Uses wrong tool (write instead of edit)
      → Tool description doesn't explain the difference
        → Error occurs
          → No reflection mechanism
            → Agent pushes forward with bad state
              → Hits iteration limit
                → Returns broken result
```

Fix any one of these and quality improves. Fix all of them and you get Claude Code-level results.

---

## 3. Deep Dive: The Agent Loop

### 3.1 Current Implementation

**File:** `crates/rusvel-agent/src/lib.rs`

The core loop (`run_streaming_loop`, lines 627-832):

```rust
// Simplified — the actual code
for iteration in 0..MAX_ITERATIONS {  // MAX_ITERATIONS = 10
    compact_messages_if_needed(&mut messages, &self.llm).await?;

    let request = build_request(&config, &messages, &tool_defs);
    let mut rx = self.llm.stream(request).await?;

    // Collect streaming response
    let response = collect_stream(&mut rx, &event_tx).await?;

    match response.finish_reason {
        FinishReason::Stop | FinishReason::Length | FinishReason::ContentFilter => {
            // Done — return output
            return Ok(AgentOutput { ... });
        }
        FinishReason::ToolUse => {
            // Extract FIRST tool call only
            let (tool_call_id, tool_name, tool_args) = extract_tool_call(&response)?;

            // Pre-hooks (can deny/modify)
            let hook_result = run_hooks_pre(&self.hooks, &tool_name, &tool_args).await;
            if let HookDecision::Deny(reason) = hook_result {
                messages.push(tool_error(tool_call_id, reason));
                continue;
            }

            // Execute tool
            let result = self.tools.call(&tool_name, effective_args).await;

            // Append result to messages
            messages.push(tool_result_message(tool_call_id, result));

            // Special: tool_search injects discovered tools
            if tool_name == "tool_search" {
                if let Some(discovered) = extract_discovered_tools(&result) {
                    tool_defs.extend(discovered);
                }
            }
        }
        _ => return Err(anyhow!("Unexpected finish reason")),
    }
}
Err(anyhow!("Max iterations reached"))
```

### 3.2 Problems Identified

#### Problem 1: MAX_ITERATIONS = 10 is too low

Claude Code has no hard iteration limit — it runs until the model stops calling tools or the context window fills. Complex tasks (refactoring across files, multi-step debugging, research-then-implement) routinely need 15-30 tool calls.

**Evidence:** The Anthropic Agent SDK docs state that a "turn" is one LLM call + tool execution round. Claude Code's `max_turns` is configurable per session. The default for coding tasks is effectively unlimited (bounded by context window, not iteration count).

**Recommendation:** Increase `MAX_ITERATIONS` to 50. Add a configurable `max_turns` field to `AgentConfig`. Keep 10 as the default for simple chat, allow override for complex workflows.

```rust
// In AgentConfig (rusvel-core/src/domain.rs)
pub struct AgentConfig {
    // ... existing fields ...
    pub max_turns: Option<u32>,  // None = use default (50)
}

// In run_streaming_loop
let max = config.max_turns.unwrap_or(50);
for iteration in 0..max {
    // ...
}
```

#### Problem 2: Only the first tool call is extracted

When Claude returns multiple tool calls in one response (e.g., "read these 3 files"), `extract_tool_call()` returns only the first `Part::ToolCall`. The rest are silently dropped. The loop continues, and the model sees the result of only one tool — it has to re-request the others.

**Evidence:** The Anthropic Agent SDK explicitly handles multiple tool calls per response. Read-only tools (Read, Glob, Grep) execute in parallel. Stateful tools (Write, Edit, Bash) execute sequentially.

**Current code (lines 711-716):**
```rust
fn extract_tool_call(response: &LlmResponse) -> Result<(String, String, Value)> {
    for part in &response.content.parts {
        if let Part::ToolCall { id, name, args } = part {
            return Ok((id.clone(), name.clone(), args.clone()));
        }
    }
    Err(anyhow!("No tool call found"))
}
```

**Recommendation:** Extract ALL tool calls. Classify tools as read-only or stateful. Execute read-only tools concurrently with `tokio::JoinSet`, stateful tools sequentially.

```rust
fn extract_all_tool_calls(response: &LlmResponse) -> Vec<(String, String, Value)> {
    response.content.parts.iter()
        .filter_map(|part| match part {
            Part::ToolCall { id, name, args } => Some((id.clone(), name.clone(), args.clone())),
            _ => None,
        })
        .collect()
}

// Read-only tools that can run in parallel
const READ_ONLY_TOOLS: &[&str] = &[
    "read_file", "glob", "grep", "git_status", "git_diff", "git_log",
    "memory_search", "memory_read", "tool_search", "code_search",
    "harvest_list", "harvest_pipeline", "content_list",
];

fn is_read_only(tool_name: &str) -> bool {
    READ_ONLY_TOOLS.contains(&tool_name)
}
```

#### Problem 3: No graceful degradation at iteration limit

When `MAX_ITERATIONS` is hit, the function returns `Err(anyhow!("Max iterations reached"))`. The user sees an error. All partial work is lost. No summary of what was accomplished.

**Recommendation:** At the iteration limit, instead of erroring, inject a system message asking the model to summarize what it accomplished and what remains:

```rust
if iteration == max - 1 {
    messages.push(LlmMessage {
        role: LlmRole::System,
        content: Content::text(
            "You are approaching your turn limit. Summarize what you have accomplished \
             so far and what remains to be done. Do not call any more tools."
        ),
    });
    // One final LLM call for summary
    let final_response = self.llm.generate(build_request(&config, &messages, &[])).await?;
    return Ok(AgentOutput::from_response(final_response));
}
```

### 3.3 Comparison: How Claude Code's Loop Differs

```
Claude Code Loop (verified from SDK docs):
┌──────────────────────────────────────────────────────┐
│  1. Receive prompt + system prompt + tools + history │
│  2. Claude responds with text and/or tool calls      │
│  3. If tool calls:                                   │
│     a. Classify: read-only vs stateful               │
│     b. Read-only → parallel execution (tokio::join!) │
│     c. Stateful → sequential execution               │
│     d. All results fed back                          │
│  4. If text only → emit final result                 │
│  5. If approaching context limit → compact           │
│  6. Repeat from 2                                    │
│  No hard iteration limit.                            │
│  Stop conditions: model stops, context full, budget. │
└──────────────────────────────────────────────────────┘
```

```
RUSVEL Loop (current):
┌──────────────────────────────────────────────────────┐
│  1. Receive prompt + system prompt + tools           │
│  2. Claude responds                                  │
│  3. If tool call (FIRST ONLY):                       │
│     a. Pre-hook check                                │
│     b. Execute single tool                           │
│     c. Result fed back                               │
│  4. If stop → return                                 │
│  5. If iteration 10 → ERROR                          │
│  6. Repeat from 2                                    │
│  Hard cap at 10 iterations.                          │
│  No parallel execution.                              │
│  No graceful degradation.                            │
└──────────────────────────────────────────────────────┘
```

---

## 4. Deep Dive: System Prompts

### 4.1 Current System Prompt Assembly

RUSVEL assembles the system prompt through multiple layers in `department.rs`:

**Layer 1 — Profile context (from `UserProfile::to_system_prompt()`):**
```
You are the RUSVEL AI assistant for {name}, {role}.
{tagline}

Primary skills: {skills}
Products: {products} — {desc}
Mission: {mission}
Values: {values}
Communication style: {style}

You help {name} with planning, strategy, development, content creation,
finding opportunities, and business operations. Be {style}.
```

**Layer 2 — Rules injection:**
```
--- Rules ---
[RuleName1]: content
[RuleName2]: content
```

**Layer 3 — Context pack (if session exists):**
```
--- Session context ---
Workspace: session_name
Goals: goal1, goal2
Recent events: event1, event2
Metrics: metric_summary
```

**Layer 4 — Capabilities overview (hard-coded text):**
```
DEPARTMENTS: forge, code, content, harvest...
WIRED ENGINE ACTIONS: code analyze, content draft...
PLATFORM CAPABILITIES: chat, agents, skills, rules...
TOOLS: 22 registered (10 always loaded, rest via tool_search)
```

**Layer 5 — RAG knowledge (if vector store wired):**
```
--- Relevant Knowledge ---
[Top 5 semantic search results from vector store]
```

**Total: ~500-800 tokens.** Compare to Claude Code's system prompt at **5000+ tokens**.

### 4.2 What's Missing

#### Missing: Reasoning Framework

Claude Code's system prompt includes explicit instructions on HOW to think:

```
# Doing tasks
- In general, do not propose changes to code you haven't read.
  If a user asks about or wants you to modify a file, read it first.
- Do not create files unless they're absolutely necessary.
  Prefer editing an existing file to creating a new one.
- If an approach fails, diagnose why before switching tactics —
  read the error, check your assumptions, try a focused fix.
  Don't retry the identical action blindly.
```

RUSVEL has none of this. The agent is told WHO it is but not HOW to work.

#### Missing: Tool Selection Guidance

Claude Code's system prompt explicitly maps situations to tools:

```
# Using your tools
- To read files use Read instead of cat, head, tail, or sed
- To edit files use Edit instead of sed or awk
- To create files use Write instead of cat with heredoc
- To search for files use Glob instead of find or ls
- To search content use Grep instead of grep or rg
```

RUSVEL agents don't know which tool to prefer in which situation.

#### Missing: Error Recovery Instructions

Claude Code's system prompt includes:

```
If an approach fails, diagnose why before switching tactics —
read the error, check your assumptions, try a focused fix.
Don't retry the identical action blindly, but don't abandon
a viable approach after a single failure either.
```

RUSVEL agents have no guidance on what to do when things go wrong.

#### Missing: Output Quality Standards

Claude Code's system prompt includes:

```
# Output efficiency
Keep your text output brief and direct.
Lead with the answer or action, not the reasoning.
Skip filler words, preamble, and unnecessary transitions.
```

RUSVEL agents tend to be verbose because nothing tells them not to be.

#### Missing: Safety Constraints

Claude Code's system prompt includes detailed safety instructions:

```
# Executing actions with care
Carefully consider the reversibility and blast radius of actions.
For actions that are hard to reverse, affect shared systems,
or could otherwise be risky — check with the user first.
```

RUSVEL agents have no concept of action risk assessment.

### 4.3 Recommended System Prompt Architecture

A layered system prompt that grows with context:

```
┌─────────────────────────────────────────────┐
│  Layer 0: Core Reasoning Framework (fixed)  │  ~800 tokens
│  HOW to think, tool selection, error        │
│  recovery, output standards, safety         │
├─────────────────────────────────────────────┤
│  Layer 1: Department Identity               │  ~200 tokens
│  WHO you are, department role, capabilities │
├─────────────────────────────────────────────┤
│  Layer 2: Department-Specific Guidance      │  ~300 tokens
│  Domain knowledge, common patterns,         │
│  department-specific tool preferences       │
├─────────────────────────────────────────────┤
│  Layer 3: Rules (user-defined)              │  ~200 tokens
│  Injected from ObjectStore                  │
├─────────────────────────────────────────────┤
│  Layer 4: Session Context                   │  ~300 tokens
│  Goals, recent events, metrics, RAG         │
├─────────────────────────────────────────────┤
│  Layer 5: Available Actions                 │  ~200 tokens
│  What tools and APIs are available          │
└─────────────────────────────────────────────┘
Total: ~2000 tokens (well within budget)
```

#### Layer 0: Core Reasoning Framework (proposed)

```markdown
# How You Work

You are an AI agent inside RUSVEL, a virtual agency platform. You have tools
to read files, write code, search codebases, execute commands, manage content,
track opportunities, and more. Use them proactively.

## Approach

1. **Understand before acting.** Read existing code/content before modifying.
   Search for patterns before creating new ones.
2. **Use the right tool.** Prefer `read_file` over `bash cat`. Prefer `edit_file`
   over `write_file` for existing files. Prefer `grep` over `bash grep`.
3. **Verify your work.** After writing code, check for errors. After creating
   content, review it against the brief. After modifying files, read the result.
4. **Recover from errors.** When a tool fails, read the error message carefully.
   Diagnose the cause. Try a different approach — don't retry the exact same call.
5. **Be concise.** Lead with the answer. Skip preamble. Show what you did, not
   what you're about to do.

## Safety

- Never execute destructive operations without confirming the intent.
- Prefer reversible actions (edit over delete, branch over force-push).
- When uncertain, explain your plan and ask before proceeding.

## Tool Selection

- **Reading files:** Use `read_file`. Set `offset` and `limit` for large files.
- **Finding files:** Use `glob` with patterns like `**/*.rs` or `src/**/*.ts`.
- **Searching content:** Use `grep` with regex. Use `glob_filter` to narrow scope.
- **Editing files:** Use `edit_file` for surgical changes. Use `write_file` only
  for new files or complete rewrites.
- **Running commands:** Use `bash` for builds, tests, installs. Set timeout for
  long operations.
- **Remembering context:** Use `memory_write` to save important facts within the
  session. Use `memory_search` to recall them later.
- **Discovering tools:** If you need a capability not in your current tools, call
  `tool_search` to find specialized tools.
```

#### Department-Specific Guidance Examples

**Code Department:**
```markdown
## Code Department Guidance

You are a senior software engineer. Your primary tools are code analysis,
search, and generation.

- Always run `code_analyze` before making architectural recommendations.
- Use `code_search` to find existing patterns before writing new code.
- When writing Rust: use `thiserror` for lib errors, `anyhow` in binaries.
- When reviewing: check for OWASP top 10, missing error handling, untested paths.
- After writing code, suggest running `cargo test` or `cargo check`.
```

**Content Department:**
```markdown
## Content Department Guidance

You are a content strategist and writer. Your tools create, adapt, and publish
content across platforms.

- Always ask about target audience and platform before drafting.
- Use `content_draft` with the right `kind` — don't use Blog for tweets.
- Adapt content for each platform's voice — LinkedIn is professional,
  Twitter is punchy, DEV.to is technical.
- Content MUST be approved before publishing. Never skip the approval step.
- Use `content_list` to check what's already in the pipeline before creating
  duplicates.
```

**Harvest Department:**
```markdown
## Harvest Department Guidance

You are a business development specialist. Your tools discover and qualify
opportunities.

- Start with `harvest_scan` to discover new opportunities.
- Score opportunities before proposing — use `harvest_score`.
- Check `harvest_pipeline` stats before deciding what to focus on.
- Proposals should be personalized — reference the opportunity details.
- Track pipeline stages: Cold → Contacted → Qualified → ProposalSent → Won/Lost.
```

### 4.4 Implementation

**Where to store the prompts:**

Option A: Embed in Rust code as `const` strings (current approach for profile).
- Pro: Compile-time checked, no I/O.
- Con: Hard to iterate, requires recompile.

Option B: Store in `DepartmentManifest` (domain type in `rusvel-core`).
- Pro: Per-department, introspectable, can be overridden via config.
- Con: More complex wiring.

**Recommendation:** Option B. Add a `reasoning_framework: String` field to `DepartmentManifest` and a global default in `rusvel-core`. The department chat handler prepends the framework to the system prompt.

```rust
// In DepartmentManifest (rusvel-core)
pub struct DepartmentManifest {
    // ... existing fields ...
    pub reasoning_framework: Option<String>,  // Department-specific guidance
    pub compaction_rules: Vec<String>,        // What to preserve during summarization
}
```

The global framework lives as a `const` in `rusvel-agent`:

```rust
pub const CORE_REASONING_FRAMEWORK: &str = include_str!("prompts/core_framework.md");
```

---

## 5. Deep Dive: Tool Descriptions

### 5.1 Current Tool Descriptions (Full Audit)

Here is every tool description currently in the codebase:

| Tool | Current Description | Token Count |
|------|-------------------|-------------|
| `read_file` | "Read the contents of a file. Returns the file text." | 12 |
| `write_file` | "Write content to a file. Creates the file if it doesn't exist, overwrites if it does." | 19 |
| `edit_file` | "Perform a search-and-replace edit on a file. The old_string must match exactly." | 16 |
| `glob` | "Find files matching a glob pattern. Returns matching file paths." | 12 |
| `grep` | "Search file contents for a regex pattern. Returns matching lines with file paths and line numbers." | 18 |
| `bash` | "Execute a bash command and return its stdout/stderr. Commands have a timeout." | 14 |
| `git_status` | "Show the working tree status (staged, unstaged, untracked files)." | 12 |
| `git_diff` | "Show changes between commits, commit and working tree, etc." | 11 |
| `git_log` | "Show recent commit history." | 5 |
| `tool_search` | "Search for available tools by keyword." | 7 |
| `memory_write` | "Store a memory entry. Returns the UUID of the stored entry." | 11 |
| `memory_read` | "Recall a specific memory entry by its UUID." | 9 |
| `memory_search` | "Search memory entries within a session using a text query. Returns matching entries." | 14 |
| `memory_delete` | "Delete a memory entry by its UUID." | 8 |
| `delegate_agent` | "Delegate a task to a sub-agent..." (longer, ~60 words) | ~80 |
| `harvest_scan` | "Scan for freelance opportunities using the mock source." | 10 |
| `harvest_score` | "Re-score an existing opportunity and update its stored score." | 11 |
| `harvest_propose` | "Generate a tailored proposal for a stored opportunity." | 9 |
| `harvest_list` | "List opportunities, optionally filtered by pipeline stage." | 9 |
| `harvest_pipeline` | "Get pipeline statistics for a session." | 7 |
| `content_draft` | "Draft new AI-generated content on a given topic." | 9 |
| `content_adapt` | "Adapt existing content for a target platform." | 8 |
| `content_publish` | "Publish approved content to a platform. Content must be approved first." | 12 |
| `content_list` | "List content items for a session, optionally filtered by status." | 11 |
| `content_approve` | "Mark a content item as human-approved (required before publishing)." | 11 |
| `code_analyze` | "Analyze a repository: parse Rust files, build symbol graph, compute metrics, and index for search." | 17 |
| `code_search` | "Search previously indexed code symbols using BM25." | 8 |

**Average: ~14 tokens per description. Claude Code averages ~200+ tokens per tool.**

### 5.2 Research-Backed Template

From the arxiv paper "Learning to Rewrite Tool Descriptions for Reliable LLM-Agent Tool Use" (2602.20426v1):

> Tool descriptions improve selection accuracy by 2-3% per tool at the subtask level — compounding significantly across multi-step chains. Agents struggle most with large candidate sets (100+ tools). Include "when NOT to use" for tools with overlapping semantics.

**Recommended template:**

```
Tool to <action>. Use when <trigger condition>.
Do NOT use when <anti-pattern>.

Parameters:
- <name> (<required|optional>): <what it represents>. Example: "<example_value>"

Returns: <format description>

Notes: <constraints, error conditions, common mistakes>
```

### 5.3 Rewritten Tool Descriptions (Priority Tools)

#### `read_file` — Before vs After

**Before:**
```
Read the contents of a file. Returns the file text.
```

**After:**
```
Read a file's contents with optional line range. Use this whenever you need
to understand existing code, check a file's current state, or verify your
changes. Always read a file before editing it.

Do NOT use bash (cat, head, tail) to read files — use this tool instead.

Parameters:
- path (required): Absolute or relative file path. Example: "src/lib.rs"
- offset (optional): Start line (1-based). Use for large files. Example: 100
- limit (optional): Max lines to read. Default: entire file. Example: 50

Returns: Line-numbered file content (like `cat -n`).

Notes:
- For files >500 lines, use offset+limit to read in chunks.
- Returns an error if the file doesn't exist — check with glob first if unsure.
- Can read images (PNG, JPG) and PDFs (use pages param for large PDFs).
```

#### `edit_file` — Before vs After

**Before:**
```
Perform a search-and-replace edit on a file. The old_string must match exactly.
```

**After:**
```
Replace a specific string in a file with new content. Use for surgical edits
to existing files — adding a function, fixing a bug, changing a value.
Prefer this over write_file for existing files (preserves unchanged content).

Do NOT use this for new files (use write_file). Do NOT use bash (sed/awk).

Parameters:
- path (required): File to edit. Example: "crates/rusvel-api/src/chat.rs"
- old_string (required): Exact text to find and replace. Must be unique in the
  file — if ambiguous, include surrounding context lines. Example: "fn main() {"
- new_string (required): Replacement text. Must differ from old_string.

Returns: Success confirmation or error.

Notes:
- FAILS if old_string appears multiple times. Include more context to disambiguate.
- FAILS if old_string isn't found. Read the file first to get exact content.
- Preserves file encoding and line endings.
```

#### `bash` — Before vs After

**Before:**
```
Execute a bash command and return its stdout/stderr. Commands have a timeout.
```

**After:**
```
Execute a shell command and return stdout + stderr. Use for builds (cargo build),
tests (cargo test), installs (pnpm install), git operations, and system commands.

Do NOT use for reading files (use read_file), searching (use grep/glob),
or editing files (use edit_file).

Parameters:
- command (required): The bash command. Example: "cargo test -p rusvel-api"
- timeout_ms (optional): Timeout in ms. Default: 120000 (2 min). Max: 600000.
  Example: 300000 for long builds.

Returns: Combined stdout/stderr output. Exit code in metadata.

Notes:
- Commands run in the project root directory.
- Long-running commands: set timeout_ms appropriately to avoid premature kill.
- For interactive commands (requiring stdin): not supported — tell the user
  to run it manually.
- Quote paths with spaces: cd "path with spaces"
```

#### `grep` — Before vs After

**Before:**
```
Search file contents for a regex pattern. Returns matching lines with file paths and line numbers.
```

**After:**
```
Search file contents for a regex pattern across the codebase. Use for finding
function definitions, imports, error messages, configuration values, or any
text pattern across multiple files.

Do NOT use bash (grep/rg) — this tool is optimized for the project structure.

Parameters:
- pattern (required): Regex pattern. Example: "fn\\s+handle_chat" or "TODO|FIXME"
- path (optional): Directory or file to search. Default: project root "."
  Example: "crates/rusvel-api/src/"
- glob_filter (optional): File pattern filter. Example: "*.rs" or "*.ts"

Returns: Matching lines with file paths and line numbers, like ripgrep output.

Notes:
- Uses ripgrep syntax (not PCRE). Escape special chars: \\{, \\}, \\(, \\)
- For case-insensitive search, use (?i) prefix: "(?i)error"
- Results may be truncated for very broad patterns — narrow with path or glob_filter.
```

### 5.4 Implementation Approach

Store enhanced descriptions in dedicated files that are included at compile time:

```
crates/rusvel-builtin-tools/src/
  descriptions/
    read_file.md
    write_file.md
    edit_file.md
    glob.md
    grep.md
    bash.md
    git_status.md
    git_diff.md
    git_log.md
    tool_search.md
```

```rust
// In file_ops.rs
ToolDefinition {
    name: "read_file".to_string(),
    description: include_str!("descriptions/read_file.md").to_string(),
    // ...
}
```

This makes descriptions easy to iterate without rewriting Rust code.

---

## 6. Deep Dive: Context Management

### 6.1 Current State

**Constants (lib.rs):**
- `COMPACT_THRESHOLD = 30` messages — triggers summarization
- `COMPACT_KEEP_RECENT = 10` messages — preserved verbatim
- Chat history limit: `50` messages loaded (chat.rs, line 83)

**Compaction logic (lib.rs, lines 528-609):**
1. Keep system message (index 0)
2. Keep last 10 messages
3. Summarize middle section with fast-tier LLM (temperature=0.2, max_tokens=2048)
4. Compaction prompt: "Preserve key facts, user goals, tool calls and outcomes, and decisions."
5. Replace old messages with single `[Earlier conversation summary]` system message

**Problems:**
1. **Information loss** — Summary is lossy. Specific file paths, error messages, partial results are compressed away.
2. **Generic compaction prompt** — Same summary instructions regardless of department. Code department should preserve file paths; content department should preserve draft status.
3. **No retrieval from summarized content** — Once summarized, details are gone. No way to retrieve specific earlier facts.
4. **50-message history cap** — Conversation reloads only last 50 messages from storage. Long sessions lose context permanently (not just summarized — gone).

### 6.2 How Claude Code Manages Context

From Anthropic's engineering blog "Effective harnesses for long-running agents":

1. **Full context accumulation** — Nothing resets between turns within a session. System prompt, tool definitions, and CLAUDE.md are prompt-cached (amortized cost).
2. **Automatic compaction** — When approaching the context limit (not a message count), the SDK summarizes older turns. A `compact_boundary` event fires.
3. **CLAUDE.md re-injection** — Persistent instructions are re-injected on every request, not subject to compaction loss. This is critical: rules and framework survive compaction.
4. **Session artifacts** — For long-running tasks, Claude Code writes structured artifacts (JSON task lists, progress files) that persist across compaction. The agent reads these at the start of each compacted turn.

### 6.3 Recommendations

#### 6.3.1 Department-Specific Compaction Instructions

Add `compaction_rules` to `DepartmentManifest`:

```rust
// Code department
compaction_rules: vec![
    "Preserve all file paths that were read or modified",
    "Preserve cargo test results and error messages",
    "Preserve architectural decisions and trade-offs discussed",
],

// Content department
compaction_rules: vec![
    "Preserve content IDs and their current status (draft/approved/published)",
    "Preserve platform targets and publishing schedule",
    "Preserve editorial feedback and revision notes",
],

// Harvest department
compaction_rules: vec![
    "Preserve opportunity IDs and their pipeline stages",
    "Preserve scoring results and reasoning",
    "Preserve proposal status and client feedback",
],
```

The compaction prompt becomes:
```
Summarize this conversation. Preserve:
- Key facts and user goals
- Tool calls and their outcomes
- Decisions made
{department_specific_rules}
```

#### 6.3.2 Re-inject Framework After Compaction

After compaction, the reasoning framework (Layer 0) should be re-injected as a system message, not rely on surviving the summary:

```rust
// After compact_messages()
if was_compacted {
    // Re-inject core framework
    messages.insert(1, LlmMessage {
        role: LlmRole::System,
        content: Content::text(CORE_REASONING_FRAMEWORK),
    });
}
```

#### 6.3.3 Memory-Backed Context Retrieval

Instead of relying solely on conversation history, use `memory_search` to retrieve relevant facts from earlier in the session:

```rust
// Before each LLM call, inject relevant memories
if let Some(memory_port) = &self.memory {
    let recent_input = extract_recent_user_input(&messages);
    let memories = memory_port.search(&config.session_id, &recent_input, 5).await?;
    if !memories.is_empty() {
        let memory_text = format_memories(&memories);
        messages.push(LlmMessage {
            role: LlmRole::System,
            content: Content::text(format!("--- Recalled context ---\n{memory_text}")),
        });
    }
}
```

#### 6.3.4 Increase History Limit

Change the chat history load from 50 to 100 messages, and make it configurable:

```rust
// In chat.rs
let history_limit = chat_config.history_limit.unwrap_or(100);
let history = load_history(&state.storage, &conversation_id, history_limit).await?;
```

### 6.4 Context Budget Allocation

As prompts grow richer, budget discipline matters:

| Component | Token Budget | Notes |
|-----------|-------------|-------|
| Core reasoning framework | 800 | Fixed, re-injected after compaction |
| Department identity + guidance | 500 | Per-department, fixed |
| Rules injection | 300 | User-defined, variable |
| Context pack (goals/events/metrics) | 400 | Session-dependent |
| RAG knowledge | 500 | Query-dependent |
| Tool definitions (non-searchable) | 1500 | ~60 tokens per tool x 10 tools |
| Conversation history | Remainder | Grows until compaction |
| **Total system overhead** | **~4000** | Leaves ~196K for conversation |

---

## 7. Deep Dive: Error Recovery & Reflection

### 7.1 Current Error Handling

When a tool fails in RUSVEL (lib.rs, lines 752-819):

```rust
let tool_result = self.tools.call(&tool_name, effective_args).await;

let (result_text, is_error) = match &tool_result {
    Ok(r) => (r.output.text().unwrap_or_default(), false),
    Err(e) => (format!("Tool error: {e}"), true),
};

messages.push(LlmMessage {
    role: LlmRole::Tool,
    content: Content::with_parts(vec![Part::ToolResult {
        tool_call_id,
        content: result_text,
        is_error,
    }]),
});
// ... continue loop, hope the model figures it out
```

**That's it.** The error is stuffed into the conversation as text. The model receives `is_error: true` but no guidance on what to do about it.

### 7.2 What Claude Code Does Differently

Claude Code's error handling is multi-layered:

1. **System prompt guidance:** "If an approach fails, diagnose why before switching tactics — read the error, check your assumptions, try a focused fix."
2. **Tool-level validation:** Parameters are validated against JSON Schema before execution. Invalid args are caught before the tool runs.
3. **Structured error context:** Error messages include actionable information (file not found → suggest glob search; permission denied → suggest checking path).
4. **Permission system:** Destructive tools require explicit approval. The agent proposes, the user confirms.

### 7.3 RUSVEL's Existing (Unwired) Verification System

**RUSVEL already has a verification chain!** It's in `crates/rusvel-agent/src/verification.rs`:

```rust
pub struct VerificationChain {
    steps: Vec<Box<dyn VerificationStep>>,
}

pub enum VerificationResult {
    Pass { confidence: f64 },
    Fail {
        confidence: f64,
        issues: Vec<String>,
        suggested_fix: Option<String>,  // ← THIS IS GOLD
    },
    Skip,
}
```

**Built-in verification steps:**
1. **`LlmCritiqueStep`** — Sends output to a cheap LLM (Haiku-class) for critique. Returns `{pass, confidence, issues}` JSON.
2. **`RulesComplianceStep`** — Checks output against regex patterns from rules.

**The problem: This chain is never called from the agent loop.** It exists as a standalone utility for external callers. The `suggested_fix` from `LlmCritiqueStep` is computed but never fed back into the agent.

### 7.4 Recommendations

#### 7.4.1 Wire Verification Chain into the Loop

Add a `verification_chain` field to `AgentConfig` and call it after the agent produces a final response:

```rust
// In run_streaming_loop, after FinishReason::Stop
let output_text = response.content.text().unwrap_or_default();

if let Some(chain) = &self.verification_chain {
    let verification = chain.run(&output_text).await?;

    if let Some((step_name, VerificationResult::Fail { issues, suggested_fix, .. })) =
        verification.iter().find(|(_, r)| matches!(r, VerificationResult::Fail { .. }))
    {
        if iteration < max_iterations - 2 {  // Leave room for fix + summary
            let fix_prompt = match suggested_fix {
                Some(fix) => format!(
                    "Your response had issues detected by {step_name}:\n{}\n\n\
                     Suggested fix: {fix}\n\n\
                     Please revise your response addressing these issues.",
                    issues.join("\n- ")
                ),
                None => format!(
                    "Your response had issues detected by {step_name}:\n{}\n\n\
                     Please revise your response addressing these issues.",
                    issues.join("\n- ")
                ),
            };

            messages.push(LlmMessage {
                role: LlmRole::User,
                content: Content::text(fix_prompt),
            });
            continue;  // Re-run the loop with fix guidance
        }
    }
}
```

#### 7.4.2 Add Error Context Enhancement

When a tool fails, enhance the error message with recovery guidance:

```rust
fn enhance_tool_error(tool_name: &str, error: &str) -> String {
    let base = format!("Tool '{tool_name}' failed: {error}");
    let guidance = match tool_name {
        "read_file" if error.contains("not found") =>
            "\nThe file doesn't exist. Use `glob` to search for similar filenames.",
        "read_file" if error.contains("permission") =>
            "\nPermission denied. Check the file path is correct.",
        "edit_file" if error.contains("not unique") =>
            "\nThe search string appears multiple times. Include more surrounding \
             context in old_string to make it unique.",
        "edit_file" if error.contains("not found") =>
            "\nThe search string wasn't found. Use `read_file` to see the current \
             file content, then retry with the exact text.",
        "bash" if error.contains("timeout") =>
            "\nCommand timed out. Try increasing timeout_ms or breaking the \
             command into smaller steps.",
        "bash" if error.contains("exit code") =>
            "\nCommand failed. Read the stderr output above to diagnose the issue.",
        _ => "",
    };
    format!("{base}{guidance}")
}
```

#### 7.4.3 Add Pre-Execution Validation

Validate tool arguments before calling the tool:

```rust
// Before tools.call()
if tool_name == "edit_file" {
    // Validate that old_string != new_string
    if args.get("old_string") == args.get("new_string") {
        let error = "old_string and new_string are identical. Nothing to change.";
        messages.push(tool_error_message(tool_call_id, error));
        continue;
    }
}

if tool_name == "write_file" || tool_name == "edit_file" {
    // Check that the path doesn't contain suspicious patterns
    if let Some(path) = args.get("path").and_then(|v| v.as_str()) {
        if path.contains("..") || path.starts_with("/etc") || path.starts_with("/usr") {
            let error = format!("Suspicious path: {path}. Refusing to modify system files.");
            messages.push(tool_error_message(tool_call_id, error));
            continue;
        }
    }
}
```

#### 7.4.4 Reflexion Pattern: Retry with Context

For high-value outputs (content drafts, proposals, code generation), add a reflection step:

```rust
pub struct ReflectionConfig {
    pub enabled: bool,
    pub max_retries: u32,        // Default: 2
    pub reflection_model: Option<ModelRef>,  // Cheap model for critique
    pub criteria: Vec<String>,   // What to check for
}
```

The reflection loop:
```
1. Agent produces output
2. Reflection model critiques output against criteria
3. If issues found AND retries remain:
   a. Inject critique as user message
   b. Continue loop (agent sees its own output + critique)
4. If no issues OR max retries reached:
   a. Return output
```

---

## 8. Deep Dive: Multi-Agent Orchestration

### 8.1 Current State

RUSVEL has two orchestration mechanisms:

**1. WorkflowRunner (rusvel-agent/src/workflow.rs):**
```rust
pub enum WorkflowStep {
    Agent { config: AgentConfig, input_mapping: Option<String> },
    Sequential { steps: Vec<WorkflowStep> },
    Parallel { steps: Vec<WorkflowStep> },
    Loop { step: Box<WorkflowStep>, max_iterations: u32, until: Option<String> },
}
```
- Static: Steps defined at workflow creation time
- The harness controls routing, not the LLM
- Output of step N feeds into step N+1

**2. delegate_agent tool (rusvel-builtin-tools/src/delegate.rs):**
- LLM-initiated delegation to sub-agents
- Max delegation depth: 3
- Sub-agent runs with its own persona, tools, model tier
- Terminal pane integration for visibility
- Result returned as text to parent agent

### 8.2 Gaps

1. **No dynamic routing** — WorkflowRunner uses static step definitions. Can't adapt based on intermediate results.
2. **No shared state** — Sub-agents don't share state with parent. Each starts fresh with only the task prompt.
3. **No result aggregation** — Parallel step results are collected but not synthesized. No "merge the results of these 3 analyses into a summary" step.
4. **No delegation context** — When `delegate_agent` is called, the sub-agent gets only the `prompt` parameter. It doesn't know the parent's goals, session context, or what other agents have done.
5. **No handoff pattern** — An agent can't say "I'm the wrong agent for this, hand off to the content department." It has to call `delegate_agent` explicitly.

### 8.3 Recommendations

#### 8.3.1 Context-Rich Delegation

Enhance the `delegate_agent` tool to pass session context:

```rust
// When building sub-agent config from delegate_agent args
let sub_config = AgentConfig {
    session_id: parent_config.session_id.clone(),  // Same session
    instructions: Some(format!(
        "{persona_instructions}\n\n\
         --- Delegation Context ---\n\
         Parent agent: {parent_persona}\n\
         Parent's goal: {parent_goal}\n\
         Your task: {task_prompt}\n\
         Session context: {context_pack_summary}"
    )),
    // ...
};
```

#### 8.3.2 Result Aggregation Step

Add a `WorkflowStep::Aggregate` variant for combining parallel results:

```rust
pub enum WorkflowStep {
    // ... existing variants ...
    Aggregate {
        steps: Vec<WorkflowStep>,      // Run in parallel
        synthesis_prompt: String,       // How to combine results
        synthesis_config: AgentConfig,  // Which agent synthesizes
    },
}
```

#### 8.3.3 Department-Aware Routing

The God Agent (global chat) should be able to route to departments without explicit `delegate_agent` calls. Add a lightweight routing layer:

```rust
// In the God Agent system prompt
"When a user's request is specific to a department, route it there:
- Code questions → delegate to Code department
- Content creation → delegate to Content department
- Opportunity hunting → delegate to Harvest department
- Financial queries → delegate to Finance department
Use delegate_agent with the appropriate department persona."
```

---

## 9. Deep Dive: Human-in-the-Loop

### 9.1 Current State

RUSVEL has a solid approval model:

```rust
pub enum ApprovalStatus { Pending, Approved, Rejected }
pub struct ApprovalPolicy {
    pub require_approval: bool,
    pub auto_approve_below: Option<f64>,  // Cost threshold
}
```

Jobs with `JobStatus::AwaitingApproval` pause until human action. Content publishing and outreach sending require approval by default.

**API routes:**
- `GET /api/approvals` — list pending approvals
- `POST /api/approvals/{id}/approve` — approve
- `POST /api/approvals/{id}/reject` — reject
- Frontend: `ApprovalQueue` component with sidebar badge

### 9.2 Gaps

1. **No approval timeout** — Approvals sit forever. No escalation, no notification.
2. **No reasoning summary** — Approvals show the action but not WHY the agent chose it.
3. **No dry-run mode** — Can't preview what the agent WOULD do without actually doing it.
4. **No inline confirmation** — Agent can't ask "Should I proceed?" during execution. It either does the action or hits a pre-defined approval gate.

### 9.3 Recommendations

#### 9.3.1 Approval Timeout & Escalation

```rust
// In rusvel-cron or job worker
async fn check_stale_approvals(storage: &dyn ObjectStore, channel: &dyn ChannelPort) {
    let stale = storage.objects().list("jobs", json!({
        "status": "AwaitingApproval",
        "created_before": Utc::now() - Duration::hours(1),
    })).await?;

    for job in stale {
        // Emit escalation event
        events.emit(Event::new("approval.escalated", &job)).await?;
        // Notify via channel (Telegram, etc.)
        channel.send(&format!(
            "Pending approval for {} hours: {}\nApprove at /approvals",
            job.age_hours(), job.description
        )).await?;
    }
}
```

#### 9.3.2 Reasoning Summary in Approvals

When an agent enqueues a job requiring approval, include its reasoning:

```rust
// In the agent's tool execution
let job = Job {
    kind: JobKind::ContentPublish,
    payload: json!({
        "content_id": content_id,
        "platform": "linkedin",
        // Agent's reasoning chain for the approval reviewer
        "reasoning": "The user asked me to publish the blog post about Rust async patterns. \
                      I adapted it for LinkedIn (shortened to 1300 chars, added hashtags). \
                      The content was previously approved as a blog draft.",
    }),
    status: JobStatus::AwaitingApproval,
    // ...
};
```

#### 9.3.3 Plan Mode (Dry Run)

Add a `PermissionMode` to `AgentConfig`:

```rust
pub enum PermissionMode {
    Default,           // Normal execution, pre-hooks can deny
    Plan,              // No tool execution — describe what would be done
    AcceptEdits,       // Auto-approve file edits
    BypassPermissions, // All tools auto-approved (CI/testing only)
}
```

In Plan mode, tool calls return descriptions instead of executing:

```rust
if config.permission_mode == PermissionMode::Plan {
    let plan_text = format!(
        "PLAN: Would call tool '{}' with args: {}",
        tool_name,
        serde_json::to_string_pretty(&tool_args)?
    );
    messages.push(tool_result_message(tool_call_id, plan_text, false));
    continue;
}
```

---

## 10. Deep Dive: Streaming & Feedback Loops

### 10.1 Current Streaming Architecture

**Events emitted (AgentEvent enum):**
```rust
TextDelta { text: String }           // Incremental LLM output
ToolCall { tool_call_id, name, args } // Tool invocation
ToolResult { tool_call_id, name, output, is_error }  // Tool result
StateDelta { delta: serde_json::Value }  // State updates
Done { output: AgentOutput }          // Completion
Error { message: String }            // Failure
```

**AG-UI wire format (SSE):**
```
event: RUN_STARTED
data: {"run_id":"...","timestamp":"..."}

event: TEXT_DELTA
data: {"text":"Hello, "}

event: TOOL_CALL_START
data: {"tool_call_id":"...","tool_name":"read_file","args":{...}}

event: TOOL_CALL_END
data: {"tool_call_id":"...","output":"...","is_error":false}

event: RUN_COMPLETED
data: {"run_id":"...","output":{...}}
```

**Problem: One-way only.** Events flow from agent to client. The client cannot:
- Interrupt the agent mid-run
- Provide feedback ("no, not that file")
- Approve/deny inline tool calls
- Redirect the agent's approach

### 10.2 Recommendations

#### 10.2.1 Cancellation Channel

Add a cancellation mechanism via the `RunState`:

```rust
pub struct RunState {
    pub config: AgentConfig,
    pub status: AgentStatus,
    pub cancel: CancellationToken,  // tokio_util::sync::CancellationToken
}

// In run_streaming_loop, check before each LLM call
if self.runs.read().await.get(&run_id)
    .map_or(true, |s| s.cancel.is_cancelled())
{
    return Ok(AgentOutput::cancelled());
}
```

**API endpoint:**
```
POST /api/dept/{dept}/chat/cancel
```

**Frontend:** "Stop" button that sends cancel request.

#### 10.2.2 Inline Approval for Sensitive Tools

For tools marked as `supervised` in permissions, emit an approval event and wait:

```rust
if permission_mode == ToolPermissionMode::Supervised {
    // Emit approval request
    event_tx.send(AgentEvent::ApprovalRequired {
        tool_call_id: tool_call_id.clone(),
        tool_name: tool_name.clone(),
        args: tool_args.clone(),
    }).await?;

    // Wait for approval (with timeout)
    match approval_rx.recv_timeout(Duration::from_secs(300)).await {
        Ok(ApprovalResponse::Approved) => { /* proceed */ }
        Ok(ApprovalResponse::Denied(reason)) => {
            messages.push(tool_error(tool_call_id, reason));
            continue;
        }
        Err(_timeout) => {
            messages.push(tool_error(tool_call_id, "Approval timed out"));
            continue;
        }
    }
}
```

#### 10.2.3 User Feedback Injection

Allow the client to inject messages into the agent's conversation mid-run:

```rust
// New API endpoint
// POST /api/dept/{dept}/chat/inject
// Body: { "run_id": "...", "message": "Actually, use the other file" }

// In AgentRuntime, check for injected messages before each LLM call
if let Some(injected) = self.check_injected_messages(&run_id).await {
    messages.push(LlmMessage {
        role: LlmRole::User,
        content: Content::text(injected),
    });
}
```

---

## 11. The Marketplace Connection

### 11.1 Why Agent Intelligence Enables the Marketplace

The marketplace vision (see `docs/plans/capability-marketplace-design.md`) depends on agent intelligence:

1. **Skills are only as good as the agent using them.** A marketplace skill "Generate SEO-optimized blog post" is a prompt template. If the agent doesn't know to research first, draft second, and verify third — the skill produces mediocre output regardless of how well the template is written.

2. **Workflows need intelligent execution.** A marketplace workflow "Code → Content Pipeline" chains `code_analyze` → `content_draft`. If the agent can't handle errors between steps, recover from failed analysis, or adapt the content prompt based on analysis results — the workflow is brittle.

3. **Rules need enforcement.** A marketplace rule "Never publish content without SEO metadata" is text in the system prompt. Without the verification chain wired in, the agent can ignore it.

4. **Discovery needs context.** The Capability Engine (`!capability`) discovers tools via WebSearch. An intelligent agent can evaluate whether discovered tools actually fit the need. A dumb agent installs everything blindly.

### 11.2 Intelligence as a Marketplace Artifact

The intelligence improvements themselves are marketplace artifacts:

| Artifact Type | Example | Marketplace Value |
|--------------|---------|-------------------|
| **Reasoning Framework** | "Code Review Framework" — step-by-step code review methodology | Teaches agents HOW to review code |
| **Tool Description Pack** | "Enhanced File Operations" — rich descriptions for all file tools | Makes agents use tools correctly |
| **Verification Chain** | "Content Quality Gate" — LLM critique + rules compliance | Auto-checks content before publishing |
| **Compaction Rules** | "Code Session Preservation" — what to keep during summarization | Prevents context loss in code sessions |
| **Reflection Config** | "High-Stakes Output Validator" — 3-retry reflection with criteria | Catches errors in important outputs |

### 11.3 Marketplace-Ready Agent Config

A marketplace artifact bundle should include agent configuration alongside the artifact itself:

```json
{
    "name": "SEO Content Pipeline",
    "type": "workflow",
    "version": "1.2.0",
    "artifacts": {
        "workflow": { "steps": [...] },
        "skills": [
            { "name": "seo-keyword-research", "prompt": "..." },
            { "name": "meta-description-writer", "prompt": "..." }
        ],
        "rules": [
            { "name": "seo-metadata-required", "content": "..." }
        ],
        "agent_config": {
            "reasoning_framework": "content-creation",
            "verification_chain": ["seo-compliance", "readability-check"],
            "reflection": { "enabled": true, "max_retries": 2 },
            "compaction_rules": ["Preserve SEO keywords", "Preserve target audience"]
        }
    }
}
```

---

## 12. Implementation Roadmap

### Phase 0: Quick Wins (1 day)

These changes require minimal code and dramatically improve agent quality:

| Change | File | Effort | Impact |
|--------|------|--------|--------|
| Increase MAX_ITERATIONS to 50 | `rusvel-agent/src/lib.rs` line 45 | 1 line | Agents can finish complex tasks |
| Add configurable `max_turns` to AgentConfig | `rusvel-core/src/domain.rs` | 5 lines | Per-task iteration control |
| Increase chat history limit to 100 | `rusvel-api/src/chat.rs` line 83 | 1 line | Better conversation context |
| Add graceful degradation at iteration limit | `rusvel-agent/src/lib.rs` | 15 lines | No more "max iterations" errors |

### Phase 1: System Prompts (1-2 days)

| Change | File | Effort | Impact |
|--------|------|--------|--------|
| Create core reasoning framework (`prompts/core_framework.md`) | New file in `rusvel-agent/src/` | 1 file | Agents know HOW to think |
| Create per-department guidance prompts | New files in `rusvel-agent/src/prompts/` | 14 files | Domain-specific intelligence |
| Add `reasoning_framework` to DepartmentManifest | `rusvel-core/src/domain.rs` | 5 lines | Customizable per department |
| Prepend framework in dept_chat handler | `rusvel-api/src/department.rs` | 10 lines | Framework reaches the agent |
| Re-inject framework after compaction | `rusvel-agent/src/lib.rs` | 10 lines | Framework survives long sessions |

### Phase 2: Tool Descriptions (1 day)

| Change | File | Effort | Impact |
|--------|------|--------|--------|
| Create `descriptions/` directory with .md files | `rusvel-builtin-tools/src/descriptions/` | 10 files | Rich tool documentation |
| Update tool registration to use `include_str!` | `rusvel-builtin-tools/src/file_ops.rs` etc. | 10 registrations | Descriptions load at compile time |
| Add enhanced descriptions for engine tools | `rusvel-engine-tools/src/` | 12 files | Engine tools get proper guidance |

### Phase 3: Parallel Tool Calls (1-2 days)

| Change | File | Effort | Impact |
|--------|------|--------|--------|
| Replace `extract_tool_call` with `extract_all_tool_calls` | `rusvel-agent/src/lib.rs` | 20 lines | Captures all tool calls |
| Add `READ_ONLY_TOOLS` classification | `rusvel-agent/src/lib.rs` | 15 lines | Parallel safety |
| Implement parallel execution with `JoinSet` | `rusvel-agent/src/lib.rs` | 40 lines | Read-only tools run concurrently |
| Update AG-UI events for parallel tool calls | `rusvel-agent/src/lib.rs` | 15 lines | UI shows parallel execution |

### Phase 4: Error Recovery (2-3 days)

| Change | File | Effort | Impact |
|--------|------|--------|--------|
| Add `enhance_tool_error()` function | `rusvel-agent/src/lib.rs` | 40 lines | Actionable error messages |
| Add pre-execution validation | `rusvel-agent/src/lib.rs` | 30 lines | Catch bad args before calling |
| Wire VerificationChain into the loop | `rusvel-agent/src/lib.rs` | 50 lines | Self-correction on output |
| Add `max_verification_retries` to AgentConfig | `rusvel-core/src/domain.rs` | 5 lines | Configurable reflection depth |
| Add `ReflectionConfig` to AgentConfig | `rusvel-core/src/domain.rs` | 15 lines | Structured reflection settings |

### Phase 5: Context Management (2 days)

| Change | File | Effort | Impact |
|--------|------|--------|--------|
| Add `compaction_rules` to DepartmentManifest | `rusvel-core/src/domain.rs` | 5 lines | Department-specific preservation |
| Update `compact_messages()` to use department rules | `rusvel-agent/src/lib.rs` | 20 lines | Better summarization |
| Add memory-backed context retrieval before LLM calls | `rusvel-agent/src/lib.rs` | 30 lines | Recall from earlier context |
| Add context budget tracking | `rusvel-agent/src/lib.rs` | 40 lines | Prevent prompt overflow |

### Phase 6: Streaming & Feedback (3-5 days)

| Change | File | Effort | Impact |
|--------|------|--------|--------|
| Add CancellationToken to RunState | `rusvel-agent/src/lib.rs` | 15 lines | Interruptible agents |
| Add cancel API endpoint | `rusvel-api/src/department.rs` | 20 lines | UI stop button works |
| Add ApprovalRequired event | `rusvel-agent/src/lib.rs` | 30 lines | Inline tool approval |
| Add message injection endpoint | `rusvel-api/src/department.rs` | 30 lines | Mid-run feedback |
| Add PermissionMode to AgentConfig | `rusvel-core/src/domain.rs` | 20 lines | Plan/dry-run mode |
| Frontend: Stop button + approval dialog | `frontend/src/lib/components/` | 100 lines | User-facing controls |

### Total Effort Estimate

| Phase | Days | Cumulative |
|-------|------|------------|
| Phase 0: Quick Wins | 0.5 | 0.5 |
| Phase 1: System Prompts | 1.5 | 2 |
| Phase 2: Tool Descriptions | 1 | 3 |
| Phase 3: Parallel Tools | 1.5 | 4.5 |
| Phase 4: Error Recovery | 2.5 | 7 |
| Phase 5: Context Management | 2 | 9 |
| Phase 6: Streaming & Feedback | 4 | 13 |

**Phases 0-2 deliver 70% of the improvement in 3 days.** System prompts + tool descriptions + higher iteration limit transform agent quality without touching the loop architecture.

---

## 13. File Impact Map

### Core Domain (`rusvel-core`)

```
crates/rusvel-core/src/domain.rs
  + AgentConfig.max_turns: Option<u32>
  + AgentConfig.max_verification_retries: Option<u32>
  + AgentConfig.reflection_config: Option<ReflectionConfig>
  + AgentConfig.permission_mode: PermissionMode
  + DepartmentManifest.reasoning_framework: Option<String>
  + DepartmentManifest.compaction_rules: Vec<String>
  + PermissionMode enum { Default, Plan, AcceptEdits, BypassPermissions }
  + ReflectionConfig struct { enabled, max_retries, model, criteria }
```

### Agent Runtime (`rusvel-agent`)

```
crates/rusvel-agent/src/lib.rs
  ~ MAX_ITERATIONS: 10 → 50
  ~ extract_tool_call → extract_all_tool_calls (parallel support)
  + enhance_tool_error() function
  + pre-execution validation
  + verification chain integration
  + memory-backed context retrieval
  + cancellation token support
  + graceful degradation at limit
  + framework re-injection after compaction

crates/rusvel-agent/src/prompts/         (NEW directory)
  + core_framework.md                     Core reasoning framework
  + dept_code.md                          Code department guidance
  + dept_content.md                       Content department guidance
  + dept_harvest.md                       Harvest department guidance
  + dept_forge.md                         Forge department guidance
  + dept_gtm.md                           GTM department guidance
  + dept_finance.md                       Finance department guidance
  + dept_product.md                       Product department guidance
  + dept_growth.md                        Growth department guidance
  + dept_distro.md                        Distribution department guidance
  + dept_legal.md                         Legal department guidance
  + dept_support.md                       Support department guidance
  + dept_infra.md                         Infrastructure department guidance
  + dept_flow.md                          Flow department guidance
```

### Built-in Tools (`rusvel-builtin-tools`)

```
crates/rusvel-builtin-tools/src/descriptions/  (NEW directory)
  + read_file.md          Enhanced read_file description
  + write_file.md         Enhanced write_file description
  + edit_file.md          Enhanced edit_file description
  + glob.md               Enhanced glob description
  + grep.md               Enhanced grep description
  + bash.md               Enhanced bash description
  + git_status.md         Enhanced git_status description
  + git_diff.md           Enhanced git_diff description
  + git_log.md            Enhanced git_log description
  + tool_search.md        Enhanced tool_search description

crates/rusvel-builtin-tools/src/file_ops.rs
  ~ Use include_str!("descriptions/read_file.md") for descriptions
  ~ Same for write_file, edit_file, glob, grep

crates/rusvel-builtin-tools/src/shell.rs
  ~ Use include_str!("descriptions/bash.md")

crates/rusvel-builtin-tools/src/git.rs
  ~ Use include_str!("descriptions/git_*.md")
```

### API Layer (`rusvel-api`)

```
crates/rusvel-api/src/chat.rs
  ~ History limit: 50 → 100

crates/rusvel-api/src/department.rs
  + Prepend reasoning framework to system prompt
  + Cancel endpoint: POST /api/dept/{dept}/chat/cancel
  + Inject endpoint: POST /api/dept/{dept}/chat/inject
```

### Frontend

```
frontend/src/lib/components/chat/
  + StopButton.svelte         Cancel running agent
  + ToolApprovalDialog.svelte Inline tool approval
  ~ DepartmentChat.svelte     Wire stop button + approval
```

---

## 14. Appendix: Full Code Audit

### A. Agent Runtime Constants

| Constant | Current | Recommended | File:Line |
|----------|---------|-------------|-----------|
| `MAX_ITERATIONS` | 10 | 50 | `rusvel-agent/src/lib.rs:45` |
| `COMPACT_THRESHOLD` | 30 | 30 (keep) | `rusvel-agent/src/lib.rs:48` |
| `COMPACT_KEEP_RECENT` | 10 | 10 (keep) | `rusvel-agent/src/lib.rs:51` |
| `MAX_DELEGATION_DEPTH` | 3 | 3 (keep) | `rusvel-builtin-tools/src/delegate.rs` |
| Chat history load | 50 | 100 | `rusvel-api/src/chat.rs:83` |
| Bash timeout | 120s | 120s (keep) | `rusvel-builtin-tools/src/shell.rs` |

### B. Tool Registration Summary

| Tool | Category | Searchable | File |
|------|----------|------------|------|
| read_file | file_ops | No | builtin-tools/src/file_ops.rs |
| write_file | file_ops | No | builtin-tools/src/file_ops.rs |
| edit_file | file_ops | No | builtin-tools/src/file_ops.rs |
| glob | file_ops | No | builtin-tools/src/file_ops.rs |
| grep | file_ops | No | builtin-tools/src/file_ops.rs |
| bash | shell | No | builtin-tools/src/shell.rs |
| git_status | git | No | builtin-tools/src/git.rs |
| git_diff | git | No | builtin-tools/src/git.rs |
| git_log | git | No | builtin-tools/src/git.rs |
| tool_search | meta | No | builtin-tools/src/tool_search.rs |
| memory_write | memory | Yes | builtin-tools/src/memory.rs |
| memory_read | memory | Yes | builtin-tools/src/memory.rs |
| memory_search | memory | Yes | builtin-tools/src/memory.rs |
| memory_delete | memory | Yes | builtin-tools/src/memory.rs |
| delegate_agent | delegation | Yes | builtin-tools/src/delegate.rs |
| terminal_open | terminal | Yes | builtin-tools/src/terminal_tools.rs |
| terminal_watch | terminal | Yes | builtin-tools/src/terminal_tools.rs |
| browser_observe | browser | Yes | builtin-tools/src/browser.rs |
| browser_search | browser | Yes | builtin-tools/src/browser.rs |
| browser_act | browser | Yes | builtin-tools/src/browser.rs |
| invoke_flow | flow | Yes | builtin-tools/src/flow.rs |
| forge_save_artifact | forge | Yes | builtin-tools/src/artifacts.rs |
| harvest_scan | harvest | No | engine-tools/src/harvest.rs |
| harvest_score | harvest | No | engine-tools/src/harvest.rs |
| harvest_propose | harvest | No | engine-tools/src/harvest.rs |
| harvest_list | harvest | No | engine-tools/src/harvest.rs |
| harvest_pipeline | harvest | No | engine-tools/src/harvest.rs |
| content_draft | content | No | engine-tools/src/content.rs |
| content_adapt | content | No | engine-tools/src/content.rs |
| content_publish | content | No | engine-tools/src/content.rs |
| content_list | content | No | engine-tools/src/content.rs |
| content_approve | content | No | engine-tools/src/content.rs |
| code_analyze | code | No | engine-tools/src/code.rs |
| code_search | code | No | engine-tools/src/code.rs |

### C. Persona Catalog

| Persona | Role | Instructions | Capabilities | Default Model |
|---------|------|-------------|--------------|---------------|
| CodeWriter | Write clean code | "Write clean, idiomatic code. Follow best practices." | CodeAnalysis, ToolUse | claude-sonnet-4 |
| Reviewer | Review code | "Review code for correctness, style, and performance." | CodeAnalysis | claude-sonnet-4 |
| Tester | Write tests | "Write comprehensive tests. Identify edge cases." | CodeAnalysis | claude-sonnet-4 |
| Debugger | Fix bugs | "Diagnose and fix bugs. Trace root causes." | CodeAnalysis, ToolUse | claude-sonnet-4 |
| Architect | Design systems | "Design systems. Evaluate trade-offs. Write ADRs." | Planning, CodeAnalysis | claude-sonnet-4 |
| Documenter | Write docs | "Write clear documentation, guides, and API references." | ContentCreation | claude-sonnet-4 |
| SecurityAuditor | Audit security | "Audit code for vulnerabilities. Recommend mitigations." | CodeAnalysis | claude-sonnet-4 |
| Refactorer | Improve code | "Improve code structure without changing behavior." | CodeAnalysis | claude-sonnet-4 |
| ContentWriter | Write content | "Write blog posts, social media content, and copy." | ContentCreation | claude-sonnet-4 |
| Researcher | Research topics | "Research topics. Summarize findings. Cite sources." | WebBrowsing, ContentCreation | claude-sonnet-4 |

**Problem:** Every persona has a 1-sentence instruction. Compare to Claude Code's 5000+ word system prompt. These personas are labels, not intelligence.

### D. System Prompt Assembly Pipeline

```
department.rs::dept_chat()
│
├─ 1. DepartmentDef.default_config.system_prompt     (registry default)
├─ 2. UserProfile::to_system_prompt()                 (persona context)
├─ 3. LayeredConfig overlay                           (stored config)
├─ 4. @agent mention override                         (agent persona/model/tools)
├─ 5. /skill resolution                               (skill prompt template)
├─ 6. Rules injection ("--- Rules ---")               (enabled rules)
├─ 7. ContextPack injection ("--- Session context ---")
│     ├─ session name
│     ├─ goal titles
│     ├─ recent events (last 10)
│     └─ metrics summary
├─ 8. RAG knowledge ("--- Relevant Knowledge ---")    (if vector store wired)
├─ 9. Platform API docs                               (webhooks, cron, jobs)
└─ 10. Engine capabilities                            (department-specific actions)

MISSING:
├─ 0. Core reasoning framework                       (HOW to think)
├─ 0.5. Department-specific guidance                  (domain patterns)
└─ Post-compaction re-injection                       (framework survives)
```

### E. Verification Chain (Existing but Unwired)

**File:** `crates/rusvel-agent/src/verification.rs`

```rust
pub trait VerificationStep: Send + Sync {
    fn name(&self) -> &str;
    async fn verify(&self, ctx: &VerificationContext, output: &str) -> Result<VerificationResult>;
}

pub struct VerificationChain {
    steps: Vec<Box<dyn VerificationStep>>,
}

impl VerificationChain {
    pub async fn run(&self, ctx: &VerificationContext, output: &str)
        -> Vec<(String, VerificationResult)>
    {
        // Runs each step sequentially
        // Collects (step_name, result) pairs
        // Does NOT retry on failure — caller must handle
    }
}

// Built-in steps:
pub struct LlmCritiqueStep {
    llm: Arc<dyn LlmPort>,
    criteria: Vec<String>,
}
// → Sends output to cheap LLM with: "Critique this output against: {criteria}"
// → Expects JSON: {pass: bool, confidence: f64, issues: [...]}
// → Returns VerificationResult::Fail { suggested_fix: Some(...) }

pub struct RulesComplianceStep {
    rules: Vec<ComplianceRule>,  // name + regex pattern
}
// → Checks output text against regex patterns
// → Returns Fail with list of violated rules
```

**Status:** Fully implemented, tested, but NEVER called from `run_streaming_loop`. The chain sits dormant. Wiring it in (Phase 4) gives RUSVEL self-correction for free.

### F. AG-UI Event Protocol

```
SSE Event Names (wire format):
  RUN_STARTED         → { run_id, timestamp }
  TEXT_DELTA           → { text }
  TOOL_CALL_START      → { tool_call_id, tool_name, args }
  TOOL_CALL_END        → { tool_call_id, tool_name, output, is_error }
  STATE_DELTA          → { delta: Value }
  STEP_STARTED         → { step_id, step_name }
  STEP_COMPLETED       → { step_id }
  RUN_COMPLETED        → { run_id, output }
  RUN_FAILED           → { run_id, error }

MISSING:
  APPROVAL_REQUIRED    → { tool_call_id, tool_name, args }
  RUN_CANCELLED        → { run_id }
  CONTEXT_COMPACTED    → { summary_length }
  REFLECTION_STARTED   → { criteria }
  REFLECTION_RESULT    → { pass, issues }
```

---

## Summary: The Path from Mechanical to Magical

The gap between "Claude Code works but my app doesn't" is not architectural — it's in the **intelligence layer** sitting on top of a sound architecture.

```
RUSVEL Today:
  Architecture ████████████████████  95% ✓
  Intelligence ████░░░░░░░░░░░░░░░░  20% ✗

After this proposal:
  Architecture ████████████████████  95% ✓
  Intelligence ████████████████░░░░  80% ✓
```

**The three highest-ROI changes:**

1. **Rich system prompts** — Tell agents HOW to think, not just WHO they are. This alone transforms output quality. (Phase 1, 1.5 days)

2. **Enhanced tool descriptions** — Tell agents WHEN and HOW to use each tool. Include examples, anti-patterns, error guidance. (Phase 2, 1 day)

3. **Wire the verification chain** — The code already exists! Connect `VerificationChain` to the agent loop for self-correction. (Phase 4, partial — 0.5 days for basic wiring)

These three changes, totaling ~3 days of work, will make RUSVEL's agents dramatically more capable — not by changing the architecture, but by filling in the intelligence that the architecture was designed to support.

---

*This document should be read alongside:*
- `docs/plans/capability-marketplace-design.md` — The marketplace vision
- `docs/design/architecture-v2.md` — RUSVEL's hexagonal architecture
- `docs/design/decisions.md` — ADRs governing the codebase
