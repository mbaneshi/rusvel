# Capability Engine: Self-Configuring AI Infrastructure

> Describe what you need → RUSVEL discovers, builds, and wires the capability.
> No manual agent/skill/workflow creation. The system builds itself.

---

## The Problem

The sprint plan has 13 steps to build CRUD for agents, skills, rules, MCP servers, hooks, teams, workflows. That's infrastructure. But the user still has to manually:
- Know what agents exist
- Know what MCP servers are available
- Write system prompts
- Configure tool permissions
- Design workflows
- Connect everything

That's still too much manual work. The real question is:

> "I need to scrape job postings from LinkedIn and score them"

And RUSVEL should figure out: I need a Playwright MCP server for scraping, a scoring agent with the right prompt, a workflow that chains scrape → parse → score → store, and rules about rate limiting. Then install all of it.

---

## What Exists Online (Queryable Sources)

| Source | What | How to Query |
|--------|------|-------------|
| [mcp.so](https://mcp.so) | 3,000+ MCP servers | REST API: `https://mcp.so/api/servers?q=` |
| [smithery.ai](https://smithery.ai) | 2,200+ MCP servers | API + npm registry |
| [anthropics/skills](https://github.com/anthropics/skills) | Official agent skills | GitHub API: raw file fetch |
| [claude.com/plugins](https://claude.com/plugins) | Plugin registry (96K+ installs top) | Plugin API |
| [npm registry](https://registry.npmjs.org) | MCP server packages (`@modelcontextprotocol/*`) | `https://registry.npmjs.org/-/v1/search?text=mcp` |
| [awesome-claude-code repos](https://github.com/rohitg00/awesome-claude-code-toolkit) | 135 agents, 35 skills, 150+ plugins | GitHub API: raw markdown parse |
| GitHub search | Any repo with SKILL.md, agent definitions | GitHub search API |

These are all fetchable at runtime via `WebFetch` / `WebSearch` or plain HTTP from the Rust backend.

---

## The Architecture: Capability Engine

One new department called **Forge** (repurpose the existing Forge concept) or a meta-layer on top of all departments. It has one job: **build capabilities on demand**.

### How It Works

```
User: "I need to monitor my competitors' pricing pages"

Capability Engine:
  1. UNDERSTAND — Parse intent: web scraping + monitoring + data extraction
  2. DISCOVER  — Search mcp.so for "web scraping", find Playwright + Firecrawl
                  Search skills for "monitoring", find scheduling patterns
                  Search agents for "data extraction", find existing templates
  3. GENERATE  — Create:
                  - McpServerDef for Playwright (transport: stdio, command: npx @anthropic/playwright-mcp)
                  - McpServerDef for Firecrawl (transport: http, url: ...)
                  - AgentProfile "price-monitor" with system prompt for extracting pricing data
                  - SkillDefinition "check-prices" with prompt template
                  - WorkflowTemplate "daily-price-check" with steps: scrape → extract → compare → alert
                  - HookDefinition to run workflow daily
                  - RuleDefinition "respect robots.txt"
  4. INSTALL   — Write all definitions to ObjectStore
  5. VERIFY    — Test the MCP connection, run a sample scrape
  6. REPORT    — "Done. I installed: 2 MCP servers, 1 agent, 1 skill, 1 workflow, 1 hook, 1 rule.
                  Run /check-prices or wait for the daily schedule."
```

### The Key Insight

The Capability Engine **is itself a Claude Code call**. It uses `claude -p` with:
- `--allowedTools WebSearch,WebFetch,Bash` — to search registries and test connections
- A system prompt that knows RUSVEL's entity schemas (AgentProfile, SkillDefinition, etc.)
- `--output-format json --json-schema` — to get structured output matching our domain types

The output is JSON arrays of entities that get inserted directly into ObjectStore.

---

## Implementation

### Backend: `crates/rusvel-api/src/capability.rs`

```rust
use crate::AppState;
use rusvel_core::domain::*;
use rusvel_llm::stream::ClaudeCliStreamer;

/// The Capability Engine system prompt.
const CAPABILITY_PROMPT: &str = r#"
You are RUSVEL's Capability Engine. When the user describes what they need,
you discover, design, and output the exact configurations to make it work.

You have access to these tools:
- WebSearch: find MCP servers, skills, agents online
- WebFetch: fetch documentation, APIs, package info
- Bash: test connections, install packages

Your output MUST be a JSON object with these arrays (include only what's needed):

{
  "agents": [{ "name": "...", "role": "...", "instructions": "...", "default_model": {"provider":"anthropic","name":"sonnet"}, "allowed_tools": [...], "capabilities": [...], "budget_limit": null, "metadata": {} }],
  "skills": [{ "name": "...", "description": "...", "prompt_template": "...", "metadata": {} }],
  "rules": [{ "name": "...", "content": "...", "enabled": true, "metadata": {} }],
  "mcp_servers": [{ "name": "...", "description": "...", "transport": "stdio", "command": "...", "args": [...], "env": {}, "enabled": true, "metadata": {} }],
  "hooks": [{ "name": "...", "event": "chat.completed", "hook_type": "command", "command": "...", "enabled": true, "metadata": {} }],
  "workflows": [{ "name": "...", "description": "...", "steps": [{"name":"...","prompt":"...","agent_id":null,"depends_on":[],"approval_required":false}], "metadata": {} }],
  "explanation": "What I built and why"
}

Search online registries:
- mcp.so for MCP servers
- npm registry for @modelcontextprotocol packages
- GitHub for agent/skill templates

Always verify MCP server packages exist before recommending them.
Always include practical system prompts in agents, not generic ones.
"#;

/// POST /api/capability/build
/// Takes a natural language description, returns what was installed.
pub async fn build_capability(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CapabilityRequest>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, (StatusCode, String)> {
    let prompt = format!(
        "{}\n\nUser request: {}\n\nCurrent department: {}\n\nRespond with the JSON configuration.",
        CAPABILITY_PROMPT,
        body.description,
        body.engine.as_deref().unwrap_or("global"),
    );

    let streamer = ClaudeCliStreamer::new();
    let args = vec![
        "--model".into(), "opus".into(),
        "--effort".into(), "max".into(),
        "--allowedTools".into(), "WebSearch WebFetch Bash".into(),
    ];
    let rx = streamer.stream_with_args(&prompt, &args);

    // Stream the response to the user, then on Done, parse JSON and install
    let storage = state.storage.clone();
    let engine = body.engine.clone();

    let stream = ReceiverStream::new(rx).map(move |event| {
        match &event {
            StreamEvent::Delta { text } => {
                // Stream thinking/progress to user
                Ok(Event::default().event("delta").data(
                    serde_json::json!({"text": text}).to_string()
                ))
            }
            StreamEvent::Done { full_text, .. } => {
                // Parse the JSON output and install everything
                let storage = storage.clone();
                let engine = engine.clone();
                let text = full_text.clone();
                tokio::spawn(async move {
                    if let Ok(result) = parse_and_install(&text, &engine, &storage).await {
                        tracing::info!("Capability Engine installed: {}", result);
                    }
                });
                Ok(Event::default().event("done").data(
                    serde_json::json!({"text": full_text}).to_string()
                ))
            }
            StreamEvent::Error { message } => {
                Ok(Event::default().event("error").data(
                    serde_json::json!({"message": message}).to_string()
                ))
            }
        }
    });

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

/// Parse the LLM output JSON and install entities into ObjectStore.
async fn parse_and_install(
    text: &str,
    engine: &Option<String>,
    storage: &Arc<dyn StoragePort>,
) -> Result<String, String> {
    // Extract JSON from the response (may be wrapped in markdown code fences)
    let json_str = extract_json(text)?;
    let bundle: CapabilityBundle = serde_json::from_str(&json_str)
        .map_err(|e| format!("Failed to parse capability bundle: {e}"))?;

    let mut installed = vec![];

    // Install agents
    for mut agent in bundle.agents {
        let id = uuid::Uuid::now_v7().to_string();
        if let Some(eng) = engine {
            agent.metadata.as_object_mut().map(|m| m.insert("engine".into(), eng.clone().into()));
        }
        storage.objects().put("agents", &id, serde_json::to_value(&agent).unwrap()).await.ok();
        installed.push(format!("agent:{}", agent.name));
    }

    // Install skills
    for mut skill in bundle.skills {
        let id = uuid::Uuid::now_v7().to_string();
        if let Some(eng) = engine {
            skill.metadata.as_object_mut().map(|m| m.insert("engine".into(), eng.clone().into()));
        }
        storage.objects().put("skills", &id, serde_json::to_value(&skill).unwrap()).await.ok();
        installed.push(format!("skill:{}", skill.name));
    }

    // Same for rules, mcp_servers, hooks, workflows...

    Ok(format!("Installed: {}", installed.join(", ")))
}

#[derive(Deserialize)]
struct CapabilityBundle {
    #[serde(default)] agents: Vec<serde_json::Value>,
    #[serde(default)] skills: Vec<serde_json::Value>,
    #[serde(default)] rules: Vec<serde_json::Value>,
    #[serde(default)] mcp_servers: Vec<serde_json::Value>,
    #[serde(default)] hooks: Vec<serde_json::Value>,
    #[serde(default)] workflows: Vec<serde_json::Value>,
    #[serde(default)] explanation: String,
}

#[derive(Deserialize)]
pub struct CapabilityRequest {
    pub description: String,
    pub engine: Option<String>,
}
```

### Route

```rust
.route("/api/capability/build", post(capability::build_capability))
```

### Frontend Integration

Every department page gets a special input mode or button: **"Build Capability"**

This could be:
1. A magic prefix in the chat input: messages starting with `/build` or `!capability` go to the Capability Engine instead of regular chat
2. A dedicated button in the department header
3. A separate "Capabilities" tab in the sidebar

When triggered:
- Shows streaming output (the Capability Engine thinking, searching, configuring)
- On completion, shows what was installed (agents, skills, etc.)
- The installed items immediately appear in the relevant tabs
- User can review and adjust before using them

```svelte
<!-- In department page sidebar, add to Actions tab -->
<button
    onclick={() => sendQuickAction('!capability: ' + capabilityInput)}
    class="w-full rounded-lg border border-dashed border-brand-500/30 bg-brand-900/10 px-3 py-2 text-left"
>
    <p class="text-xs font-medium text-brand-300">Build Capability</p>
    <p class="text-[10px] text-[var(--r-fg-subtle)]">Describe what you need, AI builds it</p>
</button>
```

### In `department_chat_handler` — detect capability requests:

```rust
// If message starts with !capability, route to capability engine
if body.message.trim_start().starts_with("!capability") {
    let description = body.message.trim_start_matches("!capability").trim_start_matches(':').trim();
    return capability::build_capability_inline(engine, description, state).await;
}
```

---

## Online Registry Integration

### MCP Server Discovery

```rust
/// Search mcp.so for servers matching a query.
async fn search_mcp_registry(query: &str) -> Vec<McpServerInfo> {
    let url = format!("https://mcp.so/api/servers?q={}&limit=10", urlencoding::encode(query));
    // This happens inside the Claude -p call via WebSearch/WebFetch tools
    // The LLM searches, evaluates, and picks the right ones
}
```

The Capability Engine doesn't need to call these APIs directly — it tells Claude to use WebSearch and WebFetch tools to find what it needs. Claude does the searching, evaluating, and selecting.

### npm Package Discovery

The LLM can search npm:
```
WebFetch https://registry.npmjs.org/-/v1/search?text=mcp+playwright&size=5
```

And verify packages exist before recommending `npx @anthropic/some-mcp-server`.

### GitHub Skills/Agents Discovery

The LLM can search GitHub:
```
WebSearch "site:github.com SKILL.md claude code agent"
WebFetch https://raw.githubusercontent.com/anthropics/skills/main/some-skill/SKILL.md
```

And adapt the content into RUSVEL's SkillDefinition format.

---

## Advanced: The Self-Improving Loop

Once the Capability Engine exists, it can improve itself:

```
User: "I want better code reviews"

Capability Engine:
  1. Searches for code review best practices, OWASP guidelines, Rust-specific patterns
  2. Creates:
     - Agent "rust-reviewer" with deep Rust review prompt
     - Agent "security-reviewer" with OWASP checklist
     - Skill "/review-pr" that chains both agents
     - Workflow "code-review" that runs both in parallel, merges findings
     - Rule "always run security review on API changes"
  3. Installs everything
  4. Next time user submits code, the rules trigger automatically
```

And it keeps compounding:

```
User: "The code reviews are missing performance checks"

Capability Engine:
  1. Searches for performance review patterns
  2. Creates Agent "perf-reviewer" with benchmark awareness
  3. Updates the "code-review" workflow to include the new agent
  4. Updates rule to also trigger on hot-path code
```

The system learns and grows through conversation.

---

## What This Means for the Sprint

The Capability Engine is **Step 0** — it replaces much of the manual CRUD work. Instead of building elaborate UI forms for creating agents, skills, etc., we build:

1. The CRUD infrastructure (Steps 1-7 in the sprint — still needed as the storage layer)
2. The Capability Engine (one endpoint + system prompt)
3. A "Build Capability" button in every department

The CRUD UI becomes a **review/edit layer** — the user checks what the AI built, tweaks if needed, deletes if wrong. The primary creation path is through the Capability Engine.

### Updated Sprint Priority

```
Phase A: Storage Layer (Steps 1-8)
  → CRUD for all entity types
  → Department pages with live data
  → MCP flag wired

Phase B: Capability Engine (NEW — this doc)
  → POST /api/capability/build endpoint
  → System prompt with schema knowledge
  → !capability prefix detection in chat
  → Auto-install into ObjectStore
  → Review/edit UI for installed items

Phase C: Advanced Features (Steps 9-13)
  → Agent teams, workflows, analytics, presets
  → These become LESS critical because the Capability Engine
    can generate teams and workflows on demand
```

The Capability Engine doesn't eliminate the CRUD — it eliminates the need for the user to manually operate the CRUD. The data layer must exist. The creation path becomes AI-driven.

---

## Examples

### Example 1: "I need to publish blog posts to DEV.to"

```json
{
  "agents": [{
    "name": "devto-publisher",
    "role": "Content publisher for DEV.to",
    "instructions": "You publish articles to DEV.to via their API. Format content in markdown with frontmatter (title, tags, published). Use the DEV.to API key from environment.",
    "default_model": {"provider": "anthropic", "name": "sonnet"},
    "allowed_tools": ["WebFetch", "Bash"],
    "capabilities": ["ContentCreation"],
    "metadata": {"engine": "content"}
  }],
  "mcp_servers": [{
    "name": "devto-api",
    "description": "DEV.to REST API access",
    "transport": "http",
    "url": "https://dev.to/api",
    "env": {"DEVTO_API_KEY": "${DEVTO_API_KEY}"},
    "enabled": true,
    "metadata": {"engine": "content"}
  }],
  "skills": [{
    "name": "publish-to-devto",
    "description": "Publish a markdown article to DEV.to",
    "prompt_template": "Take this content and publish it to DEV.to. Add appropriate tags. Set published: false for draft mode. Content:\n\n{content}",
    "metadata": {"engine": "content"}
  }],
  "rules": [{
    "name": "devto-guidelines",
    "content": "When publishing to DEV.to: always include 3-5 relevant tags, add a cover image URL if available, set canonical_url if cross-posting, keep titles under 60 chars.",
    "enabled": true,
    "metadata": {"engine": "content"}
  }],
  "explanation": "Installed DEV.to publishing capability: 1 agent (devto-publisher), 1 MCP server (devto-api), 1 skill (/publish-to-devto), 1 rule (devto-guidelines). Set DEVTO_API_KEY env var to use."
}
```

### Example 2: "Set up automated opportunity scanning for Rust freelance gigs"

```json
{
  "agents": [{
    "name": "rust-gig-scanner",
    "role": "Scans job boards for Rust freelance opportunities",
    "instructions": "You scan Upwork, LinkedIn, and GitHub Jobs for Rust freelance opportunities. Score each opportunity 1-10 based on: Rust relevance, budget, timeline feasibility, competition level. Output structured JSON with title, source, score, reasoning.",
    "default_model": {"provider": "anthropic", "name": "sonnet"},
    "allowed_tools": ["WebSearch", "WebFetch"],
    "capabilities": ["OpportunityDiscovery", "WebBrowsing"],
    "metadata": {"engine": "harvest"}
  }],
  "mcp_servers": [{
    "name": "playwright-browser",
    "description": "Browser automation for scraping job boards",
    "transport": "stdio",
    "command": "npx",
    "args": ["-y", "@anthropic/playwright-mcp"],
    "env": {},
    "enabled": true,
    "metadata": {"engine": "harvest"}
  }],
  "skills": [{
    "name": "scan-rust-gigs",
    "description": "Scan job boards for Rust freelance opportunities",
    "prompt_template": "Search Upwork, LinkedIn Jobs, and GitHub Jobs for Rust freelance opportunities posted in the last 7 days. Score each 1-10. Return top 10 results.",
    "metadata": {"engine": "harvest"}
  }],
  "workflows": [{
    "name": "daily-gig-scan",
    "description": "Daily automated scan for Rust opportunities",
    "steps": [
      {"name": "scan", "prompt": "Run /scan-rust-gigs", "agent_id": null, "depends_on": [], "approval_required": false},
      {"name": "filter", "prompt": "Filter results: only score >= 7, budget >= $1000", "depends_on": ["scan"], "approval_required": false},
      {"name": "draft-proposals", "prompt": "Draft proposals for top 3 opportunities", "depends_on": ["filter"], "approval_required": true}
    ],
    "metadata": {"engine": "harvest"}
  }],
  "hooks": [{
    "name": "daily-scan-trigger",
    "event": "schedule.daily",
    "hook_type": "command",
    "command": "curl -X POST http://localhost:3000/api/workflows/daily-gig-scan/run",
    "enabled": true,
    "metadata": {"engine": "harvest"}
  }],
  "explanation": "Installed Rust gig scanning: 1 agent (rust-gig-scanner), 1 MCP server (Playwright for scraping), 1 skill (/scan-rust-gigs), 1 workflow (daily scan → filter → draft proposals with approval), 1 daily trigger hook."
}
```

### Example 3: "I want Claude to help me with SvelteKit 5 using latest docs"

```json
{
  "mcp_servers": [{
    "name": "context7",
    "description": "Live, version-accurate documentation for any framework",
    "transport": "stdio",
    "command": "npx",
    "args": ["-y", "@anthropic/context7-mcp"],
    "env": {},
    "enabled": true,
    "metadata": {"engine": "code"}
  }],
  "rules": [{
    "name": "svelte5-patterns",
    "content": "Always use Svelte 5 patterns: $props() not 'export let', snippets not slots, $state/$derived not writable stores, $bindable() for two-way binding. Check Context7 MCP for latest SvelteKit docs before answering.",
    "enabled": true,
    "metadata": {"engine": "code"}
  }],
  "explanation": "Installed Context7 MCP server for live SvelteKit 5 docs + a rule enforcing Svelte 5 patterns. The Code department now has access to latest framework documentation."
}
```
