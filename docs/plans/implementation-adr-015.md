# Implementation Design: ADR-015 Agent Intelligence Layer

> **Sprint:** [sprint-agent-intelligence.md](sprint-agent-intelligence.md)
> **ADR:** [../design/adr-015-agent-intelligence-layer.md](../design/adr-015-agent-intelligence-layer.md)
> **Date:** 2026-03-30

---

## Table of Contents

1. [Phase A: Core Reasoning Framework](#phase-a-core-reasoning-framework)
2. [Phase B: Parallel Tool Execution](#phase-b-parallel-tool-execution)
3. [Phase C: Verification Integration](#phase-c-verification-integration)
4. [Phase D: Graceful Degradation + Permission Mode](#phase-d-graceful-degradation--permission-mode)
5. [Phase E: Tool Description Enrichment](#phase-e-tool-description-enrichment)

---

## Phase A: Core Reasoning Framework

### A1: Create `core_framework.md`

**New file:** `crates/rusvel-agent/src/prompts/core_framework.md`

```markdown
# How You Work

You are an AI agent inside RUSVEL, a virtual agency platform. You have tools
to read files, write code, search codebases, execute commands, manage content,
track opportunities, and more.

## Approach

1. **Understand before acting.** Read existing code/content before modifying.
   Search for patterns before creating new ones. Never edit a file you haven't read.
2. **Use the right tool.** Prefer `read_file` over `bash cat`. Prefer `edit_file`
   over `write_file` for existing files. Prefer `grep` over `bash grep`.
   Use `glob` to find files, not `bash find`.
3. **Verify your work.** After writing code, run tests or at minimum read the
   result. After creating content, review against the brief.
4. **Recover from errors.** When a tool fails, read the error carefully.
   Diagnose the root cause. Try a different approach — do not retry the exact
   same call. If 3 tools fail in a row, stop and reassess your entire approach.
5. **Be concise.** Lead with the answer or action. Skip preamble. Show what
   you did and why, not what you're about to do.
6. **Use memory.** Call `memory_write` to save important facts, decisions, or
   intermediate results. Call `memory_search` to recall context from earlier
   in the session.

## Tool Selection Quick Reference

| Need | Tool | NOT |
|------|------|----|
| Read a file | `read_file` | `bash cat/head/tail` |
| Find files by name | `glob` | `bash find/ls` |
| Search file contents | `grep` | `bash grep/rg` |
| Edit existing file | `edit_file` | `write_file` (overwrites) |
| Create new file | `write_file` | — |
| Run build/test/install | `bash` | — |
| Check git state | `git_status`, `git_diff` | `bash git` |
| Find more tools | `tool_search` | — |

## Safety

- Never delete files or run destructive commands without confirming intent.
- Prefer reversible actions (edit over delete, branch over force-push).
- Content must be approved before publishing — never skip approval gates.
- When uncertain about scope, explain your plan before executing.
```

### A2: Department Guidance Prompts

**New files in `crates/rusvel-agent/src/prompts/`:**

```markdown
# dept_code.md
## Code Department

You are a senior software engineer. Your tools analyze code, search symbols,
and generate implementations.

- Always run `code_analyze` on the target path before making architectural
  recommendations. Raw code reading is not enough for large codebases.
- Use `code_search` to find existing patterns before writing new code.
  Duplicating existing abstractions is worse than extending them.
- When writing Rust: `thiserror` for lib errors, `anyhow` in binaries,
  edition 2024. Check `Cargo.toml` for the crate's existing conventions.
- After writing code, suggest `cargo check` or `cargo test -p <crate>`.
- Keep crates under 2000 lines (project ADR-007).
```

```markdown
# dept_content.md
## Content Department

You are a content strategist and writer. Your tools create, adapt, and
publish content across platforms.

- Ask about target audience and platform BEFORE drafting.
- Use `content_draft` with the correct `kind` — Blog for articles,
  Tweet for Twitter, LinkedInPost for LinkedIn. Never use Blog for tweets.
- Adapt content for each platform's voice:
  LinkedIn = professional/insightful, Twitter = punchy/concise,
  DEV.to = technical/tutorial-style.
- Content MUST be approved before publishing via `content_approve`.
  Never call `content_publish` on unapproved content.
- Check `content_list` before drafting to avoid duplicating existing content.
```

```markdown
# dept_harvest.md
## Harvest Department

You are a business development specialist. Your tools discover and qualify
freelance opportunities.

- Start with `harvest_scan` to discover new opportunities.
- Always score opportunities with `harvest_score` before generating proposals.
  Low-score opportunities waste effort.
- Check `harvest_pipeline` for current pipeline stats before deciding focus.
  If Won pipeline is empty, focus on advancing existing opportunities.
- Proposals must be personalized. Use `harvest_list` to get opportunity details
  and reference specific requirements in proposals.
- Pipeline stages: Cold → Contacted → Qualified → ProposalSent → Won/Lost.
```

```markdown
# dept_forge.md
## Forge Department

You are the strategic orchestrator of the virtual agency. You coordinate
across all departments to execute missions.

- Use `delegate_agent` to assign tasks to specialist departments.
  Don't try to do code/content/harvest work yourself — delegate it.
- When planning daily missions, gather status from each department first.
- Save mission plans and reviews as artifacts using `forge_save_artifact`.
- Balance ambition with capacity — a focused plan beats a scattered one.
```

```markdown
# dept_gtm.md
## GoToMarket Department

You manage CRM, outreach sequences, and invoicing.

- Outreach emails require human approval before sending.
- Personalize outreach based on the prospect's context, not templates.
- Track all prospect interactions in the CRM via events.
- Invoice generation should include clear line items and payment terms.
```

```markdown
# dept_flow.md
## Flow Department

You build and execute DAG workflows that chain department actions.

- Flows use petgraph DAGs with 3 node types: code, condition, agent.
- Always validate flow JSON before creating — malformed graphs waste time.
- Use `invoke_flow` to execute flows, not manual step-by-step calls.
- For conditional branches, define clear true/false paths.
```

### A3: Manifest Field Extensions

**File:** `crates/rusvel-core/src/department/manifest.rs`

```rust
// Add after line 37 (system_prompt field):

    /// Structured reasoning guidance for this department's agents.
    /// Appended after the core reasoning framework in the system prompt.
    /// Use for domain-specific tool preferences, common patterns, and
    /// department-specific safety constraints.
    #[serde(default)]
    pub reasoning_guidance: String,

    /// Facts to preserve during context compaction (summarization).
    /// Injected into the compaction prompt so the summarizer knows what
    /// department-specific details matter.
    /// Examples: "file paths modified", "content IDs and status",
    /// "opportunity IDs and pipeline stages".
    #[serde(default)]
    pub compaction_preserve: Vec<String>,
```

Update `DepartmentManifest::new()` default:

```rust
    reasoning_guidance: String::new(),
    compaction_preserve: Vec::new(),
```

### A4: Wire Framework into dept_chat

**File:** `crates/rusvel-api/src/department.rs`

In the system prompt assembly section of `dept_chat`, prepend the core framework:

```rust
use rusvel_agent::CORE_REASONING_FRAMEWORK;

// After building resolved.system_prompt from profile + dept config:
let mut full_prompt = String::with_capacity(4096);

// Layer 0: Core reasoning framework (always first)
full_prompt.push_str(CORE_REASONING_FRAMEWORK);
full_prompt.push_str("\n\n");

// Layer 1: Department-specific reasoning guidance
let manifest = state.registry.get(&dept);
if let Some(m) = manifest {
    if !m.reasoning_guidance.is_empty() {
        full_prompt.push_str(&m.reasoning_guidance);
        full_prompt.push_str("\n\n");
    }
}

// Layer 2: Existing system prompt (persona, capabilities, etc.)
full_prompt.push_str(&resolved.system_prompt);

resolved.system_prompt = full_prompt;
```

### A5: Re-inject After Compaction

**File:** `crates/rusvel-agent/src/lib.rs`

In `compact_messages()`, after inserting the summary message, re-inject the framework:

```rust
async fn compact_messages(llm: &dyn LlmPort, messages: &mut Vec<LlmMessage>) {
    if messages.len() <= COMPACT_THRESHOLD {
        return;
    }

    // ... existing compaction logic ...

    // After inserting summary, re-inject core framework so it survives compaction
    let framework_msg = LlmMessage {
        role: LlmRole::System,
        content: Content::text(format!(
            "[Re-injected reasoning framework]\n{}",
            CORE_REASONING_FRAMEWORK
        )),
    };

    // Insert after system message (index 0) and summary (index 1)
    let insert_pos = if messages.first().map_or(false, |m| m.role == LlmRole::System) {
        2  // After original system + summary
    } else {
        1  // After summary
    };
    if insert_pos <= messages.len() {
        messages.insert(insert_pos, framework_msg);
    }
}
```

**Expose the framework as a public constant:**

```rust
// In crates/rusvel-agent/src/lib.rs, near the top
/// Core reasoning framework prepended to all agent system prompts.
pub const CORE_REASONING_FRAMEWORK: &str = include_str!("prompts/core_framework.md");
```

---

## Phase B: Parallel Tool Execution

### B1: Extract All Tool Calls

**File:** `crates/rusvel-agent/src/lib.rs`

Replace `extract_tool_call` (lines 339-346):

```rust
/// Extract ALL tool calls from a response (not just the first).
fn extract_all_tool_calls(
    response: &LlmResponse,
) -> Vec<(String, String, serde_json::Value)> {
    response
        .content
        .parts
        .iter()
        .filter_map(|part| match part {
            Part::ToolCall { id, name, args } => {
                Some((id.clone(), name.clone(), args.clone()))
            }
            _ => None,
        })
        .collect()
}

// Keep the old one for backwards compat in the sync `run()` path
fn extract_tool_call(response: &LlmResponse) -> Option<(String, String, serde_json::Value)> {
    Self::extract_all_tool_calls(response).into_iter().next()
}
```

### B2: Read-Only Classification

```rust
/// Check if a tool is safe to run in parallel (read-only, no side effects).
fn is_read_only_tool(tool_name: &str, tool_defs: &[ToolDefinition]) -> bool {
    // Check metadata.read_only flag (already set on file ops tools)
    if let Some(def) = tool_defs.iter().find(|t| t.name == tool_name) {
        if let Some(read_only) = def.metadata.get("read_only").and_then(|v| v.as_bool()) {
            return read_only;
        }
    }
    // Fallback: known read-only tools
    matches!(
        tool_name,
        "read_file"
            | "glob"
            | "grep"
            | "git_status"
            | "git_diff"
            | "git_log"
            | "tool_search"
            | "memory_search"
            | "memory_read"
            | "code_search"
            | "harvest_list"
            | "harvest_pipeline"
            | "content_list"
    )
}
```

### B3-B4: Parallel Dispatch + Multi-Result Messages

Replace the single-tool-call handler in `run_streaming_loop` (lines 715-842):

```rust
FinishReason::ToolUse => {
    let all_calls = Self::extract_all_tool_calls(&response);
    if all_calls.is_empty() {
        return Err(RusvelError::Agent(
            "ToolUse finish_reason but no Part::ToolCall found".into(),
        ));
    }

    // Append assistant message with ALL tool calls
    messages.push(LlmMessage {
        role: LlmRole::Assistant,
        content: response.content,
    });

    // Partition into read-only (parallel) and stateful (sequential)
    let (read_only, stateful): (Vec<_>, Vec<_>) = all_calls
        .into_iter()
        .partition(|(_, name, _)| Self::is_read_only_tool(name, &tool_defs));

    let mut all_results: Vec<(String, String, String, bool)> = Vec::new();

    // Execute read-only tools in parallel
    if !read_only.is_empty() {
        let mut join_set = tokio::task::JoinSet::new();
        for (call_id, name, args) in read_only {
            let tools = Arc::clone(tools);
            let hooks = hooks.to_vec();
            let tx = tx.clone();

            // Emit ToolCall event
            let _ = tx
                .send(AgentEvent::ToolCall {
                    tool_call_id: call_id.clone(),
                    name: name.clone(),
                    args: args.clone(),
                })
                .await;

            join_set.spawn(async move {
                let effective_args = match run_hooks_pre(&hooks, &name, &args) {
                    Ok(a) => a,
                    Err(reason) => {
                        return (call_id, name, format!("Tool denied: {reason}"), true);
                    }
                };
                let result = tools.call(&name, effective_args).await;
                run_hooks_post(&hooks, &name, &args);
                let (text, is_err) = match &result {
                    Ok(r) => {
                        let t = r.output.parts.iter()
                            .filter_map(|p| match p { Part::Text(t) => Some(t.as_str()), _ => None })
                            .collect::<Vec<_>>().join("");
                        (t, !r.success)
                    }
                    Err(e) => (format!("Tool error: {e}"), true),
                };
                (call_id, name, text, is_err)
            });
        }
        while let Some(Ok(result)) = join_set.join_next().await {
            all_results.push(result);
        }
    }

    // Execute stateful tools sequentially
    for (call_id, name, args) in stateful {
        let _ = tx
            .send(AgentEvent::ToolCall {
                tool_call_id: call_id.clone(),
                name: name.clone(),
                args: args.clone(),
            })
            .await;

        let effective_args = match run_hooks_pre(hooks, &name, &args) {
            Ok(a) => a,
            Err(reason) => {
                all_results.push((call_id, name, format!("Tool denied: {reason}"), true));
                continue;
            }
        };
        let result = tools.call(&name, effective_args).await;
        run_hooks_post(hooks, &name, &args);

        // Handle tool_search deferred loading
        if name == "tool_search" {
            if let Ok(ref r) = result {
                if let Some(names) = r.metadata.get("discovered_tools") {
                    if let Some(arr) = names.as_array() {
                        let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
                        let discovered = tools.search(query, arr.len().max(10));
                        for tool in discovered {
                            if !tool_defs.iter().any(|t| t.name == tool.name) {
                                tool_defs.push(tool);
                            }
                        }
                    }
                }
            }
        }

        if let Ok(ref r) = result {
            emit_tool_ag_ui_state_deltas(tx, &r.metadata).await;
        }

        let (text, is_err) = match &result {
            Ok(r) => {
                let t = r.output.parts.iter()
                    .filter_map(|p| match p { Part::Text(t) => Some(t.as_str()), _ => None })
                    .collect::<Vec<_>>().join("");
                (t, !r.success)
            }
            Err(e) => (format!("Tool error: {e}"), true),
        };
        all_results.push((call_id, name, text, is_err));
    }

    // Emit ToolResult events and build messages
    tool_calls += all_results.len() as u32;
    let mut has_error = false;

    for (call_id, name, text, is_err) in &all_results {
        let _ = tx
            .send(AgentEvent::ToolResult {
                tool_call_id: call_id.clone(),
                name: name.clone(),
                output: text.clone(),
                is_error: *is_err,
            })
            .await;
        if *is_err {
            has_error = true;
        }
    }

    // Build single Tool message with all results
    let parts: Vec<Part> = all_results
        .into_iter()
        .map(|(call_id, _, text, is_err)| Part::ToolResult {
            tool_call_id: call_id,
            content: text,
            is_error: is_err,
        })
        .collect();

    messages.push(LlmMessage {
        role: LlmRole::Tool,
        content: Content { parts },
    });

    // Error tracking for reflection
    if has_error {
        consecutive_failures += 1;
        if consecutive_failures >= 3 {
            messages.push(LlmMessage {
                role: LlmRole::System,
                content: Content::text(
                    "[REFLECTION REQUIRED] 3 consecutive tool failures. \
                     Stop and reassess your approach. What assumption is wrong? \
                     Try a completely different strategy."
                ),
            });
        }
    } else {
        consecutive_failures = 0;
    }
}
```

---

## Phase C: Verification Integration

### C1: Add Verification Chain to AgentRuntime

**File:** `crates/rusvel-agent/src/lib.rs`

```rust
pub struct AgentRuntime {
    llm: Arc<dyn LlmPort>,
    tools: Arc<dyn ToolPort>,
    memory: Arc<dyn MemoryPort>,
    runs: RwLock<HashMap<RunId, RunState>>,
    hooks: RwLock<Vec<(ToolHookConfig, HookHandler)>>,
    // NEW: Optional verification chain for output self-correction
    verification_chain: Option<Arc<VerificationChain>>,
}

impl AgentRuntime {
    pub fn new(
        llm: Arc<dyn LlmPort>,
        tools: Arc<dyn ToolPort>,
        memory: Arc<dyn MemoryPort>,
    ) -> Self {
        Self {
            llm,
            tools,
            memory,
            runs: RwLock::new(HashMap::new()),
            hooks: RwLock::new(Vec::new()),
            verification_chain: None,
        }
    }

    /// Set the verification chain for output self-correction.
    pub fn with_verification(mut self, chain: VerificationChain) -> Self {
        self.verification_chain = Some(Arc::new(chain));
        self
    }
}
```

### C2: Wire Into Loop

Add verification after `FinishReason::Stop` in `run_streaming_loop`:

```rust
FinishReason::Stop | FinishReason::Length | FinishReason::ContentFilter => {
    let output_text = response.content.text_concat();

    // Run verification chain if configured
    let max_retries = config
        .metadata
        .get("max_verification_retries")
        .and_then(|v| v.as_u64())
        .unwrap_or(1) as u32;

    if let Some(ref chain) = verification_chain {
        if iteration < max_iter.saturating_sub(2) {
            let ctx = VerificationContext {
                department_id: config
                    .metadata
                    .get(RUSVEL_META_DEPARTMENT_ID)
                    .and_then(|v| v.as_str())
                    .unwrap_or("global")
                    .to_string(),
                tool_name: None,
                original_prompt: messages
                    .iter()
                    .find(|m| m.role == LlmRole::User)
                    .map(|m| m.content.text_concat())
                    .unwrap_or_default(),
            };

            let _ = tx
                .send(AgentEvent::StateDelta {
                    delta: serde_json::json!({"verification": "running"}),
                })
                .await;

            let results = chain.run(&ctx, &output_text).await?;
            let failed = results
                .iter()
                .find(|(_, r)| r.is_fail());

            if let Some((step_name, VerificationResult::Fail { issues, suggested_fix })) = failed {
                if verification_retries < max_retries {
                    verification_retries += 1;

                    let fix_prompt = format!(
                        "Your response was reviewed by '{}' and had issues:\n{}\n\n{}\n\nPlease revise.",
                        step_name,
                        issues.iter().map(|i| format!("- {i}")).collect::<Vec<_>>().join("\n"),
                        suggested_fix.as_deref().unwrap_or("Address the issues above."),
                    );

                    // Append the original response + fix request
                    messages.push(LlmMessage {
                        role: LlmRole::Assistant,
                        content: response.content,
                    });
                    messages.push(LlmMessage {
                        role: LlmRole::User,
                        content: Content::text(fix_prompt),
                    });

                    let _ = tx
                        .send(AgentEvent::StateDelta {
                            delta: serde_json::json!({
                                "verification": "retry",
                                "issues": issues,
                                "attempt": verification_retries,
                            }),
                        })
                        .await;

                    continue;  // Re-run the loop
                }
            }
        }
    }

    // Return the output (possibly after verification passed)
    return Ok(AgentOutput {
        run_id: *run_id,
        content: response.content,
        tool_calls,
        usage: total_usage,
        cost_estimate: 0.0,
        max_iterations: None,
        metadata: serde_json::json!({}),
    });
}
```

Add `verification_retries` counter at the top of the function:

```rust
let mut verification_retries: u32 = 0;
// Pass verification_chain reference into the function
let verification_chain = &self.verification_chain;
```

### C3: AgentConfig Extension

**File:** `crates/rusvel-core/src/domain.rs`

```rust
pub struct AgentConfig {
    pub profile_id: Option<AgentProfileId>,
    pub session_id: SessionId,
    pub model: Option<ModelRef>,
    pub tools: Vec<String>,
    pub instructions: Option<String>,
    pub budget_limit: Option<f64>,
    #[serde(default)]
    pub max_iterations: Option<u32>,
    /// Maximum verification retries on output failure. Default: 1.
    /// Set to 0 to disable verification. Set higher for critical outputs.
    #[serde(default)]
    pub max_verification_retries: Option<u32>,
    /// Permission mode: Default, Plan (dry-run), Supervised (all tools need approval).
    #[serde(default)]
    pub permission_mode: PermissionMode,
    pub metadata: serde_json::Value,
}
```

### C5: Default Chain Wiring

**File:** `crates/rusvel-app/src/main.rs` (or wherever `AgentRuntime` is constructed)

```rust
use rusvel_agent::{VerificationChain, LlmCritiqueStep};

// Build verification chain with Haiku for cheap critique
let critique_model = ModelRef {
    provider: Provider::Claude,
    model: "claude-haiku-4-5-20251001".into(),
};
let chain = VerificationChain::new()
    .add(Arc::new(LlmCritiqueStep::new(Arc::clone(&llm), critique_model)));

let agent_runtime = AgentRuntime::new(
    Arc::clone(&llm),
    Arc::clone(&tools),
    Arc::clone(&memory),
)
.with_verification(chain);
```

---

## Phase D: Graceful Degradation + Permission Mode

### D1: Graceful Degradation

**File:** `crates/rusvel-agent/src/lib.rs`

Replace the error at loop end (line 851-853):

```rust
    // Last iteration — ask for summary instead of erroring
    if iteration == max_iter - 1 {
        messages.push(LlmMessage {
            role: LlmRole::System,
            content: Content::text(
                "You have reached your tool-call limit for this request. \
                 Do NOT call any more tools. Instead, provide a concise summary:\n\
                 1. What you accomplished so far\n\
                 2. What remains to be done\n\
                 3. Specific next steps the user should take"
            ),
        });
        // Fall through to next iteration which will get a Stop response
    }
} // end of for loop

// If we somehow get here (shouldn't after the summary injection), return error
Err(RusvelError::Agent(format!(
    "agent loop exceeded {max_iter} iterations without producing output"
)))
```

### D2-D3: Permission Mode

**File:** `crates/rusvel-core/src/domain.rs`

```rust
/// Controls how the agent handles tool execution permissions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PermissionMode {
    /// Normal execution. Pre-hooks can deny specific tools.
    #[default]
    Default,
    /// Dry-run mode. Tools return descriptions of what WOULD happen
    /// without actually executing. Useful for previewing agent plans.
    Plan,
    /// All tool calls emit an ApprovalRequired event and wait.
    /// Use for high-stakes workflows requiring human oversight.
    Supervised,
}
```

**File:** `crates/rusvel-agent/src/lib.rs`

In the tool execution section, before calling `tools.call()`:

```rust
// Plan mode: describe what would happen without executing
if config.permission_mode == PermissionMode::Plan {
    let plan_text = format!(
        "[PLAN MODE] Would call tool '{}' with arguments:\n{}",
        tool_name,
        serde_json::to_string_pretty(&effective_args)
            .unwrap_or_else(|_| format!("{effective_args:?}")),
    );

    let _ = tx
        .send(AgentEvent::ToolResult {
            tool_call_id: tool_call_id.clone(),
            name: tool_name.clone(),
            output: plan_text.clone(),
            is_error: false,
        })
        .await;

    messages.push(LlmMessage {
        role: LlmRole::Tool,
        content: Content {
            parts: vec![Part::ToolResult {
                tool_call_id,
                content: plan_text,
                is_error: false,
            }],
        },
    });

    tool_calls += 1;
    continue;
}
```

### D4: Cancellation Token

```rust
use tokio_util::sync::CancellationToken;

struct RunState {
    config: AgentConfig,
    status: AgentStatus,
    cancel: CancellationToken,
}

// In create():
let state = RunState {
    config,
    status: AgentStatus::Idle,
    cancel: CancellationToken::new(),
};

// In run_streaming_loop, at the top of each iteration:
if cancel_token.is_cancelled() {
    let _ = tx
        .send(AgentEvent::Error {
            message: "Run cancelled by user".into(),
        })
        .await;
    return Ok(AgentOutput {
        run_id: *run_id,
        content: Content::text("Run was cancelled."),
        tool_calls,
        usage: total_usage,
        cost_estimate: 0.0,
        max_iterations: None,
        metadata: serde_json::json!({"cancelled": true}),
    });
}

// In stop():
async fn stop(&self, run_id: &RunId) -> Result<()> {
    let mut runs = self.runs.write().await;
    if let Some(state) = runs.get_mut(run_id) {
        state.status = AgentStatus::Stopped;
        state.cancel.cancel();  // Signal the loop to stop
    }
    Ok(())
}
```

### D5: Cancel API Endpoint

**File:** `crates/rusvel-api/src/department.rs`

```rust
pub async fn dept_chat_cancel(
    State(state): State<Arc<AppState>>,
    Path(dept): Path<String>,
    Json(body): Json<CancelRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let agent = state.agent.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let run_id = body.run_id.parse::<RunId>()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    agent.stop(&run_id).await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(Json(serde_json::json!({"status": "cancelled"})))
}

#[derive(Deserialize)]
pub struct CancelRequest {
    pub run_id: String,
}
```

Route registration:

```rust
.route("/api/dept/:id/chat/cancel", post(dept_chat_cancel))
```

---

## Phase E: Tool Description Enrichment

### Audit Checklist

Tools already enhanced (have WHEN TO USE sections):
- `read_file` — good
- `write_file` — good
- `edit_file` — good
- `glob` — good
- `grep` — good

Tools needing enhancement:

**`bash` (shell.rs):**
```rust
description: "Execute a shell command and return stdout + stderr.\n\n\
    WHEN TO USE: Running builds (cargo build/test), installs (pnpm install), \
    git operations, system commands, anything that needs a shell.\n\
    WHEN NOT TO USE: Reading files (use read_file), searching (use grep/glob), \
    editing files (use edit_file).\n\n\
    TIPS:\n\
    - Default timeout: 120s. Set timeout_ms higher for long builds.\n\
    - Exit code is in metadata — check it to verify success.\n\
    - Interactive commands (requiring stdin) are not supported.\n\
    - Quote paths with spaces: cd \"path with spaces\"".into(),
```

**`git_status` (git.rs):**
```rust
description: "Show working tree status: staged, unstaged, and untracked files.\n\n\
    WHEN TO USE: Before committing, after editing files, checking repo state.\n\
    WHEN NOT TO USE: For file content changes (use git_diff).\n\n\
    TIPS:\n\
    - Returns both staged and unstaged changes.\n\
    - Untracked files are listed separately.\n\
    - Use git_diff to see actual content of changes.".into(),
```

**`git_diff` (git.rs):**
```rust
description: "Show content changes between working tree and HEAD (or staged changes).\n\n\
    WHEN TO USE: Reviewing what changed before committing, understanding modifications.\n\
    WHEN NOT TO USE: For file status overview (use git_status).\n\n\
    TIPS:\n\
    - Default: unstaged changes. Set staged=true for staged changes.\n\
    - Output is in unified diff format.\n\
    - For specific file: set path parameter.".into(),
```

**`memory_write` (memory.rs):**
```rust
description: "Store a fact, decision, or important context in session memory.\n\n\
    WHEN TO USE: Saving important intermediate results, recording decisions, \
    preserving context that might be lost during conversation compaction.\n\
    WHEN NOT TO USE: For temporary values within a single tool chain.\n\n\
    TIPS:\n\
    - Use kind='decision' for choices made, 'fact' for discovered information.\n\
    - Memory is scoped to the session — other sessions can't see it.\n\
    - Returns UUID — save it if you need to read back later.".into(),
```

**`memory_search` (memory.rs):**
```rust
description: "Search session memory for relevant facts, decisions, and context.\n\n\
    WHEN TO USE: Recalling earlier context, finding decisions made, checking \
    what was discovered in previous tool calls.\n\
    WHEN NOT TO USE: For searching file contents (use grep).\n\n\
    TIPS:\n\
    - Uses full-text search (FTS5) — natural language queries work.\n\
    - Default limit is 10 results. Narrow with specific queries.\n\
    - Results are ordered by relevance.".into(),
```

---

## Test Plan

### Unit Tests (rusvel-agent)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_all_tool_calls_returns_multiple() {
        let response = LlmResponse {
            content: Content {
                parts: vec![
                    Part::Text("Let me read both files.".into()),
                    Part::ToolCall {
                        id: "tc1".into(),
                        name: "read_file".into(),
                        args: serde_json::json!({"path": "a.rs"}),
                    },
                    Part::ToolCall {
                        id: "tc2".into(),
                        name: "read_file".into(),
                        args: serde_json::json!({"path": "b.rs"}),
                    },
                ],
            },
            finish_reason: FinishReason::ToolUse,
            usage: LlmUsage::default(),
            metadata: serde_json::json!({}),
        };
        let calls = AgentRuntime::extract_all_tool_calls(&response);
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].1, "read_file");
        assert_eq!(calls[1].1, "read_file");
    }

    #[test]
    fn is_read_only_tool_classifies_correctly() {
        let defs = vec![
            ToolDefinition {
                name: "read_file".into(),
                description: String::new(),
                parameters: serde_json::json!({}),
                searchable: false,
                metadata: serde_json::json!({"read_only": true}),
            },
            ToolDefinition {
                name: "write_file".into(),
                description: String::new(),
                parameters: serde_json::json!({}),
                searchable: false,
                metadata: serde_json::json!({"read_only": false}),
            },
        ];
        assert!(AgentRuntime::is_read_only_tool("read_file", &defs));
        assert!(!AgentRuntime::is_read_only_tool("write_file", &defs));
        assert!(AgentRuntime::is_read_only_tool("glob", &defs));  // fallback list
        assert!(!AgentRuntime::is_read_only_tool("bash", &defs)); // not in list
    }

    #[test]
    fn permission_mode_default_is_default() {
        let mode: PermissionMode = serde_json::from_str("\"default\"").unwrap();
        assert_eq!(mode, PermissionMode::Default);

        // Default deserialization
        let config: AgentConfig = serde_json::from_str(r#"{
            "session_id": "test",
            "tools": [],
            "metadata": {}
        }"#).unwrap();
        assert_eq!(config.permission_mode, PermissionMode::Default);
    }
}
```

### Integration Tests

```rust
// Test: verification chain retries on failure
#[tokio::test]
async fn verification_retry_on_fail() {
    // Setup: mock LLM that returns "bad output" first, "good output" second
    // Setup: verification chain with rules compliance that rejects "bad"
    // Assert: final output is "good output"
    // Assert: tool_calls includes verification retry
}

// Test: graceful degradation at limit
#[tokio::test]
async fn graceful_degradation_at_limit() {
    // Setup: agent with max_iterations=2, LLM always returns ToolUse
    // Assert: final response is a summary, not an error
    // Assert: summary mentions what was accomplished
}

// Test: plan mode doesn't execute tools
#[tokio::test]
async fn plan_mode_no_execution() {
    // Setup: agent with permission_mode=Plan
    // Assert: ToolResult events contain "[PLAN MODE]" prefix
    // Assert: no actual file operations occurred
}

// Test: cancellation stops the loop
#[tokio::test]
async fn cancellation_stops_loop() {
    // Setup: agent running, cancel after first tool call
    // Assert: agent returns with cancelled metadata
    // Assert: status is Stopped
}
```

---

## Migration Notes

### Backwards Compatibility

All changes are additive:
- `reasoning_guidance` and `compaction_preserve` default to empty (no behavior change)
- `max_verification_retries` defaults to `None` (verification runs with default 1 retry if chain is set)
- `permission_mode` defaults to `Default` (existing behavior)
- `extract_tool_call()` still works for the sync `run()` path
- Existing `AgentConfig` JSON without new fields deserializes correctly via `#[serde(default)]`

### Cargo.toml Changes

```toml
# crates/rusvel-agent/Cargo.toml — add:
tokio-util = { version = "0.7", features = ["sync"] }  # CancellationToken
```

No other dependency changes needed.

---

## Dependency Graph

```
Phase A (reasoning) ──┐
                      ├──→ Phase C (verification) ──→ Done
Phase E (tools)    ──┘         │
                               ▼
Phase B (parallel) ──→ Phase D (permissions) ──→ Done
```

Phases A + E can run in parallel (no dependencies).
Phase B is independent.
Phase C should follow A (verification uses the reasoning framework context).
Phase D is independent.
