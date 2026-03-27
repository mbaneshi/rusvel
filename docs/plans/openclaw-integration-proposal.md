# Proposal: OpenClaw Patterns Integration — The Full Steal

> OpenClaw is a 296K-object, 84-extension, 24-channel, 28-provider personal AI gateway.
> This proposal catalogs EVERYTHING worth porting into RUSVEL's Rust hexagonal architecture.

**Date:** 2026-03-27
**Status:** Proposal
**Scope:** 15 workstreams across all 5 roadmap phases
**Depends on:** `docs/state-report/` (7-chapter codebase audit)
**Compliant with:** `docs/design/ui-redesign-final.md` (5-zone icon rail layout) + `docs/design/ui-design-principles.md` (10 design principles)

---

## Grounding: What the State Report Tells Us

Before stealing anything, we must acknowledge where we actually are:

| Fact | Implication |
|------|------------|
| **13 engines fully implemented** — no stubs | New features plug into real business logic, not placeholders |
| **Only 3/13 depts have tools wired** (forge, code, content) | **Priority 0:** Wire 10 remaining departments BEFORE adding new capabilities |
| **Job queue is volatile** (in-memory Vec) | Must persist before adding channel delivery workers |
| **No auth middleware** on API routes | Must add before exposing webhook endpoints to the internet |
| **52,560 Rust LOC, 399 tests, 105 routes** | Mature foundation — new work is extension, not rewrite |
| **66 Svelte components, 10,623 frontend LOC** | Frontend is feature-rich; 5-zone icon rail restructuring underway |

### Compliance: 5-Zone Icon Rail Layout (`ui-redesign-final.md`)

The final UI design restructures from 3-column (sidebar+panel+chat) to 5 zones:
```
┌──────┬──────────────┬──────────────────────────┬──────────────────┐
│ ICON │  SECTION     │  MAIN CONTENT            │  CONTEXT PANEL   │
│ RAIL │  SIDEBAR     │  (scrollable)            │  (collapsible)   │
│ 48px │  200px       │  flexible                │  320px           │
│      │              │                          │                  │
│ Home │ Dept sections│  Full-width content      │  AI chat         │
│ Chat │ from manifest│  for selected section    │  or properties   │
│ ...  │              │                          │  or exec output  │
│──────│              │                          │                  │
│Forge │              │                          │                  │
│ Code │              │                          │                  │
│ ...  │              │                          │                  │
├──────┴──────────────┴──────────────────────────┴──────────────────┤
│  BOTTOM PANEL (collapsible): Terminal / Jobs / Events             │
└───────────────────────────────────────────────────────────────────┘
URL: /dept/forge/skills  (bookmarkable)
```

**All new frontend work in this proposal MUST follow this layout:**
- New global pages (Inbox, Voice, Doctor) → icon rail top section
- New department sections (Channels, Cron) → section sidebar (manifest-driven)
- Canvas content → main content area or context panel
- No new features in the retired DepartmentPanel pattern
- All new pages get bookmarkable URLs
- Context panel = quick AI chat alongside any section (Cursor model)

---

## Table of Contents

1. [Plugin SDK & Extension Architecture](#1-plugin-sdk--extension-architecture)
2. [Channel Adapters & Multi-Channel Inbox](#2-channel-adapters--multi-channel-inbox)
3. [Message Routing, Session Scoping & DM Security](#3-message-routing-session-scoping--dm-security)
4. [Hook System (20+ Lifecycle Events)](#4-hook-system-20-lifecycle-events)
5. [Agent Runtime Enhancements](#5-agent-runtime-enhancements)
6. [Memory Backend & Context Pruning](#6-memory-backend--context-pruning)
7. [Voice: STT, TTS, Wake Word, Talk Mode](#7-voice-stt-tts-wake-word-talk-mode)
8. [Canvas & A2UI Protocol](#8-canvas--a2ui-protocol)
9. [Cron, Webhooks & Automation Triggers](#9-cron-webhooks--automation-triggers)
10. [Skills Platform & ClawHub Marketplace](#10-skills-platform--clawhub-marketplace)
11. [Provider Failover & Auth Profiles](#11-provider-failover--auth-profiles)
12. [Browser Automation Enhancements](#12-browser-automation-enhancements)
13. [Native Companion Apps](#13-native-companion-apps)
14. [Onboarding, Doctor & Diagnostics](#14-onboarding-doctor--diagnostics)
15. [Deployment: Docker, Fly.io, Daemon Install](#15-deployment-docker-flyio-daemon-install)
16. [Implementation Roadmap](#16-implementation-roadmap)

---

## 0. Priority Zero — Wire the 10 Unwired Departments

> **The state report's #1 finding:** All 13 engines are fully implemented with real business logic.
> But only forge, code, and content have agent tools registered. The other 10 have working
> engines that agents simply cannot invoke. This is the single biggest leverage gap.

**This MUST be done before any OpenClaw integration.** No point adding channels, voice, or canvas
if agents can only talk to 3 of 13 departments.

### What Needs Wiring (Per Department)

| Department | Engine Methods to Wire as Tools | Est. Tools |
|-----------|--------------------------------|-----------|
| **dept-harvest** | scan_sources, score_opportunity, generate_proposal, pipeline_status, list_opportunities | 5 |
| **dept-flow** | create_flow, run_flow, list_flows, execution_status, resume_execution | 5 |
| **dept-gtm** | add_contact, create_deal, send_outreach, pipeline_status, create_invoice | 5 |
| **dept-finance** | record_transaction, ledger_status, tax_estimate, runway_forecast, expense_report | 5 |
| **dept-product** | add_roadmap_item, update_pricing, collect_feedback, prioritize_backlog | 4 |
| **dept-growth** | track_funnel, analyze_cohort, record_kpi, growth_dashboard | 4 |
| **dept-distro** | create_listing, seo_audit, add_affiliate, distro_status | 4 |
| **dept-legal** | draft_contract, compliance_check, register_ip, legal_status | 4 |
| **dept-support** | create_ticket, resolve_ticket, search_kb, nps_survey, support_status | 5 |
| **dept-infra** | deploy_service, create_monitor, report_incident, infra_status | 4 |

**Pattern is established** (forge-engine wires 5 tools in ~60 lines):
```rust
ctx.tools.register(
    ToolDefinition {
        name: "harvest_scan_sources".into(),
        description: "Scan configured sources for new opportunities".into(),
        parameters: json!({ "type": "object", "properties": { ... } }),
        searchable: true,  // deferred loading via tool_search
    },
    |args| {
        let engine = ENGINE.get().unwrap();
        Box::pin(async move { engine.scan_sources(args).await })
    },
);
```

**Also wire:**
- Event handlers (e.g., harvest listens to `code.analyzed`)
- Job handlers (e.g., `OutreachSend` → `GtmEngine.outreach()`)
- Approval gates (ADR-008) on outreach, publishing, spending

### Deliverables

- [ ] Wire ~45 tools across 10 department crates
- [ ] Wire event handlers (cross-department reactions)
- [ ] Wire job handlers (OutreachSend, HarvestScan, etc.)
- [ ] Add approval gates per ADR-008
- [ ] Persist job queue to SQLite (replace volatile Vec<Job>)
- [ ] Tests for each department's tool registration

**Estimated effort:** ~1800 lines across 10 dept-* crates + rusvel-jobs fix.

---

## 1. Plugin SDK & Extension Architecture

### What OpenClaw Has

OpenClaw's entire ecosystem runs on a plugin SDK. **84 extensions** as npm workspace packages, each registering capabilities via a unified API:

```typescript
// OpenClaw pattern: definePluginEntry()
export default definePluginEntry({
  id: "anthropic",
  name: "Anthropic Provider",
  register(api) {
    api.registerProvider(buildAnthropicProvider());
    api.registerSpeechProvider(buildAnthropicSpeechProvider());
    api.registerMediaUnderstandingProvider(anthropicMUProvider);
    api.registerImageGenerationProvider(anthropicImageGen);
    api.registerCliBackend(buildAnthropicCliBackend());
    api.registerCommand({ id: "anthropic-usage", execute: ... });
    api.registerHook({ event: "before_model_resolve", handler: ... });
  },
});
```

**One `register()` function registers:** providers, tools, hooks, channels, CLI commands, HTTP routes, gateway methods, services, conversation bindings. Everything is a plugin.

**Extension boundaries are enforced:**
- Extensions import ONLY via `openclaw/plugin-sdk/*` (never core `src/`)
- No relative imports outside `extensions/<id>/`
- Runtime deps in `dependencies` (not workspace)

### What RUSVEL Has Today

`DepartmentApp` trait with `manifest()` + `register()`. Good foundation but limited:
- Only departments can register (not arbitrary plugins)
- No provider registration (LLM providers are hardcoded in `rusvel-llm`)
- No channel registration
- No speech/media/image provider registration
- No CLI command registration from plugins
- No HTTP route registration from plugins

### What We Build

**New: `rusvel-plugin-sdk` crate** — generalized plugin system that wraps DepartmentApp

```rust
pub trait PluginEntry: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn kind(&self) -> PluginKind; // Department, Provider, Channel, Tool, Service
    fn register(&self, api: &mut PluginApi) -> Result<()>;
    fn shutdown(&self) -> BoxFuture<'static, Result<()>> { /* default no-op */ }
}

pub struct PluginApi {
    // Existing (from RegistrationContext)
    pub tools: ToolRegistrar,
    pub event_handlers: EventHandlerRegistrar,
    pub job_handlers: JobHandlerRegistrar,

    // NEW registrars
    pub providers: ProviderRegistrar,       // LLM, speech, embedding, image-gen
    pub channels: ChannelRegistrar,         // Messaging channels
    pub hooks: HookRegistrar,              // Lifecycle hooks (20+ events)
    pub commands: CommandRegistrar,         // CLI commands
    pub routes: RouteRegistrar,            // HTTP routes (Axum)
    pub services: ServiceRegistrar,        // Background services
    pub skills: SkillRegistrar,            // Skill templates
}
```

**DepartmentApp becomes syntactic sugar** over PluginEntry with `kind: Department`.

**Plugin loading:**
- Compile-time: Rust crates (current model, keep it)
- Runtime: Dynamic `.so`/`.dylib` via `libloading` (future Phase 5)
- Config: `plugins.toml` declares which plugins are enabled

### Deliverables

- [ ] `rusvel-plugin-sdk` crate (~600 lines) — PluginEntry trait, PluginApi, registrars
- [ ] Migrate DepartmentApp to implement PluginEntry (backward compatible)
- [ ] ProviderRegistrar — register LLM/speech/embedding/image providers dynamically
- [ ] ChannelRegistrar — register channel adapters
- [ ] HookRegistrar — register lifecycle hooks
- [ ] CommandRegistrar — register CLI subcommands
- [ ] RouteRegistrar — register Axum routes
- [ ] Plugin diagnostics: status, enabled, error, tool/hook/channel counts
- [ ] `plugins.toml` config file for enable/disable

---

## 2. Channel Adapters & Multi-Channel Inbox

### What OpenClaw Has

24+ channels with a modular adapter pattern. Each channel is a `ChannelPlugin` with composable adapters:

```typescript
type ChannelPlugin<ResolvedAccount> = {
  id: ChannelId;
  meta: ChannelMeta;                     // name, icon, color
  capabilities: ChannelCapabilities;      // threading, media types, reactions, polls
  setupWizard?: ChannelSetupWizard;      // interactive setup flow
  config: ChannelConfigAdapter;           // parse/validate config
  pairing?: ChannelPairingAdapter;       // link external accounts
  security?: ChannelSecurityAdapter;     // DM policies, allowlists
  outbound?: ChannelOutboundAdapter;     // send messages
  groups?: ChannelGroupAdapter;          // group chat handling
  mentions?: ChannelMentionAdapter;      // @mention detection
  status?: ChannelStatusAdapter;         // health checks
  auth?: ChannelAuthAdapter;             // OAuth, token refresh
  lifecycle?: ChannelLifecycleAdapter;   // start/stop/reload
  commands?: ChannelCommandAdapter;      // channel-specific commands
  gateway?: ChannelGatewayAdapter;       // gateway method handlers
};
```

**Message payloads are rich:**

```typescript
type ReplyPayload = {
  text?: string;
  thinking?: string;              // reasoning trace
  media?: Array<MediaItem>;       // images, audio, video, files
  poll?: PollInput;               // native polls
  components?: StructuredComponents; // buttons, cards
  mentions?: MentionInfo[];
  thread?: { replyToId, threadId };
  metadata?: Record<string, unknown>;
};
```

**Key capabilities per channel:**
- Message chunking (platform-specific limits: Twitter 280, Slack 4000, etc.)
- Media transcoding (resize, format convert)
- Threading (map external threads to sessions)
- Typing indicators
- Read receipts
- Reactions
- File uploads
- Rich formatting (Markdown → Slack blocks, Discord embeds, etc.)

### What We Build

**New: `rusvel-channel` crate** + **`ChannelPort` trait**

```rust
// In rusvel-core/src/ports.rs
#[async_trait]
pub trait ChannelPort: Send + Sync {
    async fn send(&self, msg: OutboundMessage) -> Result<DeliveryId>;
    async fn receive(&self) -> Result<Vec<InboundMessage>>;
    async fn delivery_status(&self, id: &DeliveryId) -> Result<DeliveryStatus>;
    async fn channels(&self) -> Result<Vec<ChannelInfo>>;
    async fn connect(&self, config: ChannelConfig) -> Result<()>;
    async fn disconnect(&self, channel_id: &str) -> Result<()>;
}

// Channel adapter trait (internal to rusvel-channel)
#[async_trait]
pub trait ChannelAdapter: Send + Sync {
    fn id(&self) -> &str;
    fn capabilities(&self) -> ChannelCapabilities;
    fn max_message_length(&self) -> Option<usize>;
    fn supported_media(&self) -> Vec<MediaType>;
    fn supports_threading(&self) -> bool;
    fn supports_reactions(&self) -> bool;
    async fn send(&self, msg: &OutboundMessage) -> Result<DeliveryId>;
    async fn start(&self, tx: mpsc::Sender<InboundMessage>) -> Result<()>;
    async fn stop(&self) -> Result<()>;
    async fn health(&self) -> Result<ChannelHealth>;
}

// Rich message payload (reuses existing Content type)
pub struct OutboundMessage {
    pub channel: String,
    pub recipient: String,
    pub content: Content,                          // existing Part enum
    pub thread_id: Option<String>,
    pub reply_to: Option<String>,
    pub session_id: Option<SessionId>,
    pub formatting: Option<FormattingHint>,         // markdown, plain, rich
    pub chunking: ChunkStrategy,                    // auto, manual, none
    pub metadata: serde_json::Value,
}

pub struct InboundMessage {
    pub id: String,
    pub channel: String,
    pub sender: SenderInfo,
    pub content: Content,
    pub thread_id: Option<String>,
    pub received_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

pub struct SenderInfo {
    pub id: String,
    pub name: Option<String>,
    pub channel_handle: Option<String>,             // @username, phone#, email
    pub avatar_url: Option<String>,
}
```

**Priority adapters (6 channels for solo builder):**

| Channel | Rust Crate | Use Case |
|---------|-----------|----------|
| Email (SMTP/IMAP) | `lettre` + `async-imap` | GTM outreach, invoices, notifications |
| Slack | `reqwest` (Web API) | Team/client comms, support |
| Telegram | `teloxide` | Personal assistant (like OpenClaw primary) |
| Discord | `serenity` | Community, support |
| SMS (Twilio) | `reqwest` (REST API) | Alerts, 2FA, notifications |
| WhatsApp | TBD | Client communication |

**Unified Inbox:**

```rust
pub struct InboxItem {
    pub id: String,
    pub session_id: SessionId,
    pub channel: String,
    pub sender: SenderInfo,
    pub preview: String,
    pub unread_count: u32,
    pub last_message_at: DateTime<Utc>,
    pub department: Option<String>,
    pub status: InboxStatus,  // Active, Snoozed, Archived, AwaitingReply
    pub labels: Vec<String>,
    pub metadata: serde_json::Value,
}
```

### Deliverables

- [ ] `ChannelPort` trait in rusvel-core
- [ ] `rusvel-channel` crate — adapter registry, message routing, chunking, delivery tracking
- [ ] Email adapter (SMTP + IMAP IDLE push)
- [ ] Slack adapter (Web API + Events API webhook)
- [ ] Telegram adapter (teloxide, long polling)
- [ ] Discord adapter (serenity, gateway WebSocket)
- [ ] Twilio SMS adapter
- [ ] Message chunking engine (platform-aware splitting with continuation markers)
- [ ] Media transcoding pipeline (resize images, convert formats per channel)
- [ ] Thread mapping (external thread IDs ↔ RUSVEL Sessions)
- [ ] Typing indicator forwarding
- [ ] `GET /api/inbox` — unified inbox
- [ ] `POST /api/channels/{channel}/send` — send message
- [ ] `GET /api/channels` — list connected channels
- [ ] `POST /api/channels/{channel}/connect` — configure credentials
- [ ] Frontend: Inbox page, channel config, conversation view

---

## 3. Message Routing, Session Scoping & DM Security

### What OpenClaw Has

**Sophisticated session scoping** — 4 levels of conversation isolation:

```typescript
type DmScope =
  | "main"                    // All DMs share one session
  | "per-peer"                // Each sender gets their own session
  | "per-channel-peer"        // Same sender on different channels = different sessions
  | "per-account-channel-peer"; // Most isolated: per bot account per channel per sender
```

**Session keys** encode routing:
```
agent:<agentId>:channel:<channelId>:<peer>
```

**DM security policies** (per channel):

```typescript
type DmPolicy = "pairing" | "allowlist" | "open" | "disabled";

// Pairing flow:
// 1. Unknown sender messages bot
// 2. Bot responds with 6-digit pairing code
// 3. Owner runs: openclaw pairing approve <channel> <code>
// 4. Sender added to allowlist, message reprocessed
```

**Group policies:**

```typescript
type GroupPolicy = "open" | "disabled" | "allowlist";
// + mention gating: bot only responds when @mentioned
// + reply-to tagging: bot tags original sender in replies
```

**Multi-agent routing** — route specific channels/senders to specific agents:

```typescript
// Config: different agents for different purposes
agents: {
  "support-agent": { channels: ["slack"], model: "claude-sonnet" },
  "sales-agent":   { channels: ["whatsapp", "email"], model: "claude-opus" },
  "personal":      { channels: ["telegram"], model: "claude-opus" },
}
```

### What We Build

```rust
// In rusvel-core domain types
pub enum SessionScope {
    Global,              // All inbound = one session
    PerSender,           // Each sender gets own session
    PerChannelSender,    // Same sender, different channel = different session
    PerDepartmentSender, // Routed department + sender = unique session
}

pub enum DmPolicy {
    Open,
    Pairing,
    AllowlistOnly,
    Disabled,
}

pub struct ChannelRoutingRule {
    pub channel: Option<String>,           // "slack", "telegram", "*"
    pub sender_pattern: Option<String>,    // regex or exact match
    pub keyword_pattern: Option<String>,   // message content match
    pub target_department: String,         // "support", "gtm", "forge"
    pub priority: u32,
    pub session_scope: SessionScope,
    pub dm_policy: DmPolicy,
}

pub struct PairingCode {
    pub code: String,
    pub channel: String,
    pub sender_id: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

// CLI command
// rusvel pairing approve <channel> <code>
// rusvel pairing list
// rusvel pairing revoke <channel> <sender_id>
```

**Routing flow:**

```
Inbound message arrives
    ↓
DM policy check (open? pairing? allowlist?)
    ↓ (if pairing needed, send code and stop)
    ↓ (if allowed, continue)
Match routing rules (channel + sender + keywords)
    ↓
Resolve target department
    ↓
Resolve session (by scope: global/per-sender/per-channel-sender)
    ↓
Create or resume Session + Thread
    ↓
AgentRuntime processes with department system prompt
    ↓
Response routed back through same channel
```

### Deliverables

- [ ] SessionScope enum + ChannelRoutingRule in rusvel-core
- [ ] DmPolicy + PairingCode + Allowlist in rusvel-core
- [ ] Routing engine in rusvel-channel (rule matching, department resolution)
- [ ] `rusvel pairing approve/list/revoke` CLI commands
- [ ] `POST /api/channels/routing-rules` — CRUD for routing rules
- [ ] `GET /api/channels/allowlist` — manage sender allowlists
- [ ] Per-channel session scope configuration
- [ ] Multi-agent routing: channel → department → agent mapping
- [ ] Frontend: Routing rules editor, allowlist manager

---

## 4. Hook System (20+ Lifecycle Events)

### What OpenClaw Has

**20+ hook events** spanning the entire lifecycle:

```typescript
// Agent lifecycle
before_agent_start    // Before agent turn begins
before_model_resolve  // Before model selection (can override model)
before_prompt_build   // Before system prompt assembly (can inject content)
before_dispatch       // Before sending to LLM
before_tool_call      // Before tool execution (can deny/modify)
after_tool_call       // After tool returns (can modify result)
after_compaction       // After context compaction

// Message lifecycle
inbound_claim         // Claim/reject inbound message routing
message_received      // After message parsed and routed
message_sending       // Before outbound message sent
message_sent          // After delivery confirmed

// LLM lifecycle
llm_input             // Before LLM request (inspect/modify)
llm_output            // After LLM response (inspect/modify)

// Session lifecycle
session_start         // Session created/resumed
session_end           // Session closed/expired

// Subagent lifecycle
subagent_spawning     // Before subagent spawn
subagent_spawned      // After subagent started
subagent_delivery     // Subagent result delivery

// Gateway lifecycle
gateway_start         // Gateway booted
gateway_stop          // Gateway shutting down
```

**Hook execution features:**
- Priority ordering (higher runs first)
- Merge strategies: first-defined wins for overrides, last-defined for system prompts
- Error catching with configurable throw/log behavior
- Short-circuit: e.g., `inbound_claim` with handled status stops further processing
- Async support

**Hooks can:**
- Override model selection
- Inject system prompt content
- Block/modify tool calls
- Transform LLM input/output
- Claim inbound messages for specific departments
- Execute arbitrary shell commands
- POST to webhooks
- Run Claude prompts

### What RUSVEL Has Today

RUSVEL has hooks stored in ObjectStore with 15 event types and 3 action types (command, http, prompt). But:
- No priority ordering
- No merge strategies
- No hook composition (multiple hooks on same event)
- No short-circuit/claim semantics
- No before/after LLM hooks
- No before/after tool hooks (agent runtime has its own separate hook system)
- No subagent lifecycle hooks

### What We Build

```rust
// Unify agent runtime hooks + API hooks into one system
pub enum HookEvent {
    // Agent lifecycle
    BeforeAgentStart { session_id: SessionId, department: String },
    BeforeModelResolve { requested_model: String },
    BeforePromptBuild { system_prompt: String },
    BeforeDispatch { messages: Vec<LlmMessage> },
    BeforeToolCall { tool_name: String, args: serde_json::Value },
    AfterToolCall { tool_name: String, result: ToolResult },
    AfterCompaction { original_turns: usize, compacted_turns: usize },

    // Message lifecycle
    InboundClaim { channel: String, sender: String, content: Content },
    MessageReceived { channel: String, sender: String, content: Content },
    MessageSending { channel: String, recipient: String, content: Content },
    MessageSent { channel: String, delivery_id: DeliveryId },

    // LLM lifecycle
    LlmInput { model: String, messages: Vec<LlmMessage> },
    LlmOutput { model: String, response: Content, usage: TokenUsage },

    // Session lifecycle
    SessionStart { session_id: SessionId },
    SessionEnd { session_id: SessionId },

    // Subagent lifecycle (when we add delegation)
    SubagentSpawning { parent_run: RunId, child_config: AgentConfig },
    SubagentSpawned { parent_run: RunId, child_run: RunId },
    SubagentEnded { parent_run: RunId, child_run: RunId, status: RunStatus },

    // System lifecycle
    GatewayStart,
    GatewayStop,
}

pub enum HookOutcome {
    Continue,                              // proceed normally
    Override(serde_json::Value),           // replace the input
    Deny(String),                          // block the action
    Claim { department: String },          // route to specific department
}

pub struct HookRegistration {
    pub id: String,
    pub event: HookEvent,
    pub priority: i32,                     // higher = runs first
    pub handler: HookHandler,
    pub error_policy: ErrorPolicy,         // Throw | Log | Ignore
}

pub enum HookHandler {
    Sync(Arc<dyn Fn(&HookContext) -> HookOutcome + Send + Sync>),
    Async(Arc<dyn Fn(HookContext) -> BoxFuture<'static, HookOutcome> + Send + Sync>),
    Shell { command: String },
    Http { url: String, method: String },
    Prompt { template: String },
}
```

### Deliverables

- [ ] Unified HookEvent enum (20+ events) in rusvel-core
- [ ] HookRunner with priority ordering, merge strategies, short-circuit
- [ ] Merge agent runtime hooks (ToolHookConfig) into unified system
- [ ] BeforeModelResolve hook (override model selection per department/context)
- [ ] BeforePromptBuild hook (inject system prompt sections)
- [ ] LlmInput/LlmOutput hooks (logging, transformation, guardrails)
- [ ] InboundClaim hook (department routing for channels)
- [ ] Error policy per hook (throw/log/ignore)
- [ ] Hook diagnostics (execution count, latency, errors)
- [ ] Frontend: Hook editor with event picker, priority slider, test button

---

## 5. Agent Runtime Enhancements

### What OpenClaw Has

**Pi agent runtime** with advanced features:

1. **Auth profile rotation** — multiple API keys per provider, automatic failover:
   ```typescript
   authProfiles: [
     { id: "primary", provider: "anthropic", apiKey: "sk-..." },
     { id: "backup", provider: "openai", apiKey: "sk-..." },
   ]
   // If primary rate-limited → auto-switch to backup
   ```

2. **Thinking/reasoning levels** — configurable depth:
   ```typescript
   thinkingLevel: "minimal" | "low" | "medium" | "high"
   // Maps to extended thinking budgets
   ```

3. **Session branching** — fork conversations:
   - Archive current transcript
   - Start fresh with summary context
   - `forkedFromParent` tracking

4. **Subagent spawning** — agent-to-agent delegation:
   ```typescript
   sessions_spawn tool:
     - spawnDepth tracking (0=main, 1=sub, 2=sub-sub)
     - delivery targeting (channel, account, thread)
     - status tracking (running, done, failed, killed, timeout)
     - subagentControlScope for permission boundaries
   ```

5. **Tool argument repair** — fix malformed JSON from providers:
   - XAI, Anthropic sometimes emit broken tool call args
   - Auto-repair before execution

6. **Context compaction** with retry:
   - Aggregate timeout for compaction attempts
   - Head/tail trimming for oversized tool results
   - Image soft-removal during pruning

7. **Cost tracking per session:**
   ```typescript
   inputTokens, outputTokens, totalTokens, estimatedCostUsd
   ```

### What RUSVEL Has Today

AgentRuntime with streaming, 10-iteration max, 30-turn compaction, hooks. Missing:
- No auth profile rotation / failover
- No thinking level control
- No session branching/forking
- No subagent spawning (noted in project memory)
- No tool argument repair
- No per-session cost tracking (MetricStore exists but not wired to agent)

### What We Build

```rust
// Auth profile failover
pub struct AuthProfile {
    pub id: String,
    pub provider: String,
    pub priority: u32,
    pub rate_limit_backoff: Duration,
    pub credential_handle: CredentialHandle,
}

pub struct FailoverPolicy {
    pub profiles: Vec<AuthProfile>,
    pub retry_on: Vec<FailoverTrigger>,  // RateLimit, Timeout, ServerError, Overloaded
    pub max_failovers: u32,
}

// Thinking levels
pub enum ThinkingLevel {
    Minimal,   // No extended thinking
    Low,       // Brief reasoning
    Medium,    // Standard reasoning
    High,      // Deep reasoning (maps to extended thinking budget)
}

// Session forking
impl AgentRuntime {
    pub async fn fork_session(&self, run_id: &RunId, summary: &str) -> Result<RunId>;
}

// Subagent spawning
pub struct SubagentConfig {
    pub parent_run: RunId,
    pub department: String,
    pub prompt: String,
    pub model_override: Option<String>,
    pub max_depth: u32,           // prevent infinite recursion
    pub control_scope: ControlScope,
    pub timeout: Duration,
}

pub enum ControlScope {
    Full,              // subagent has all parent tools
    ReadOnly,          // subagent can read but not write
    Scoped(Vec<String>), // specific tool allowlist
}

// Tool argument repair
pub fn repair_tool_args(raw: &str) -> Result<serde_json::Value> {
    // Try parse as-is
    // If fails: strip trailing commas, fix quotes, balance braces
    // If still fails: return error
}
```

### Deliverables

- [ ] Auth profile failover with automatic rotation on rate limit/timeout
- [ ] ThinkingLevel enum mapped to extended thinking budgets per provider
- [ ] Session fork/branch with transcript archival
- [ ] Subagent spawning (delegate_agent tool) with depth tracking
- [ ] SubagentControlScope for permission boundaries
- [ ] Tool argument repair (malformed JSON recovery)
- [ ] Per-session cost accumulation wired to MetricStore
- [ ] `rusvel agent spawn <dept> <prompt>` CLI command
- [ ] Subagent status tracking (running/done/failed/killed/timeout)

---

## 6. Memory Backend & Context Pruning

### What OpenClaw Has

**Dual memory backends:**
- **Builtin** — SQLite-based with FTS
- **QMD** — Vector database with semantic search (3 search modes: query, search, vsearch)

**Memory features:**
- Configurable flush intervals with debouncing
- Boot sync (load memory at session start)
- Session-based memory export with retention (30d default)
- Max results, snippet chars, injected chars limits
- Timeout enforcement
- Citations mode (auto/on/off) — attribute recalled memories in responses

**Context pruning algorithm:**
- Character-based token estimation (4 chars/token, 8000 chars/image)
- Head/tail trimming for oversized tool results with "..." marker
- Assistant message protection (keep last N)
- First user message boundary preservation
- Image soft-removal: `[image removed during context pruning]`
- Context window override per model

**Memory prompt sections** — dynamically built based on available tools:
- Memory search results injected as system prompt sections
- Flush plan determines when accumulated context → persistent memory

### What RUSVEL Has Today

`rusvel-memory` with FTS5 + `rusvel-vector` with LanceDB. Missing:
- No context pruning algorithm (just compaction at 30 turns)
- No memory citations
- No flush planning / debounced persistence
- No token estimation
- No image pruning during context management
- No configurable memory injection limits

### What We Build

```rust
pub struct ContextPruner {
    pub chars_per_token: usize,           // default 4
    pub chars_per_image: usize,           // default 8000
    pub max_tool_result_chars: usize,     // trim oversized tool results
    pub protected_assistant_count: usize, // keep last N assistant messages
    pub preserve_first_user: bool,
}

impl ContextPruner {
    pub fn estimate_tokens(&self, messages: &[LlmMessage]) -> usize;
    pub fn prune(&self, messages: &mut Vec<LlmMessage>, budget: usize) -> PruneReport;
}

pub struct PruneReport {
    pub messages_removed: usize,
    pub images_removed: usize,
    pub tool_results_trimmed: usize,
    pub tokens_before: usize,
    pub tokens_after: usize,
}

// Memory citations
pub struct MemoryCitation {
    pub memory_id: String,
    pub snippet: String,
    pub relevance_score: f32,
}

// Memory flush planning
pub struct FlushPlan {
    pub debounce: Duration,
    pub flush_after_turns: usize,
    pub max_injected_chars: usize,
    pub max_search_results: usize,
    pub retention: Duration,
}
```

### Deliverables

- [ ] ContextPruner with token estimation + intelligent pruning
- [ ] Head/tail trimming for tool results exceeding threshold
- [ ] Image soft-removal during pruning
- [ ] Memory citations in agent responses
- [ ] Flush planning with debounce + turn-based triggers
- [ ] Configurable memory injection limits (max chars, max results)
- [ ] Memory retention policies (auto-expire old entries)
- [ ] Memory export per session

---

## 7. Voice: STT, TTS, Wake Word, Talk Mode

### What OpenClaw Has

**Full voice pipeline:**

1. **Voice Wake** (macOS/iOS) — wake word detection:
   - Uses platform Speech.framework
   - Custom wake phrases ("Hey OpenClaw")
   - Transitions to Talk Mode on detection

2. **Talk Mode** — continuous voice conversation:
   - Real-time STT (Whisper/Deepgram)
   - TTS synthesis (ElevenLabs, system TTS fallback)
   - Overlay UI on macOS
   - Full-duplex: listen while speaking

3. **Voice Call** — telephony integration:
   - WebRTC/SIP gateway
   - Real-time audio streaming via WebSocket
   - Call routing to agents

4. **TTS Provider Plugin System:**
   ```typescript
   api.registerSpeechProvider({
     id: "elevenlabs",
     synthesize: async (text, voice) => audioBuffer,
     voices: async () => voiceList,
     streamSynthesize: async (text, voice) => readableStream,
   });
   ```

5. **Voice messages in channels:**
   - Telegram/WhatsApp voice notes → auto-transcribe → process → TTS response

### What We Build

**New: `VoicePort` trait + `rusvel-voice` crate**

```rust
#[async_trait]
pub trait VoicePort: Send + Sync {
    // STT
    async fn transcribe(&self, audio: &[u8], config: TranscribeConfig) -> Result<Transcript>;
    async fn stream_transcribe(&self, rx: mpsc::Receiver<Vec<u8>>) -> Result<mpsc::Receiver<TranscriptDelta>>;

    // TTS
    async fn synthesize(&self, text: &str, config: SynthesizeConfig) -> Result<Vec<u8>>;
    async fn stream_synthesize(&self, text: &str, config: SynthesizeConfig) -> Result<mpsc::Receiver<Vec<u8>>>;

    // Capabilities
    async fn voices(&self) -> Result<Vec<VoiceInfo>>;
    fn supports_streaming_stt(&self) -> bool;
    fn supports_streaming_tts(&self) -> bool;
}

pub struct TranscribeConfig {
    pub format: AudioFormat,       // wav, mp3, ogg, webm
    pub language: Option<String>,
    pub model: Option<String>,     // "whisper-large-v3", "deepgram-nova-2"
}

pub struct SynthesizeConfig {
    pub voice_id: String,
    pub format: AudioFormat,
    pub speed: f32,
    pub pitch: f32,
}

// Voice providers
pub enum VoiceProvider {
    OpenAiWhisper,        // STT + TTS
    ElevenLabs,           // Premium TTS + streaming
    Deepgram,             // Real-time STT
    WhisperCpp,           // Local STT (via ort)
    EdgeTts,              // Free TTS
    SystemTts,            // macOS say, Linux espeak
}
```

**Talk Mode CLI:**
```bash
rusvel talk                          # Start voice conversation
rusvel talk --wake "hey rusvel"      # Wake word mode
rusvel talk --voice elevenlabs       # Specific TTS provider
rusvel talk --dept forge             # Route to specific department
```

### Deliverables

- [ ] `VoicePort` trait in rusvel-core
- [ ] `rusvel-voice` crate with provider registry
- [ ] OpenAI Whisper adapter (STT + TTS)
- [ ] ElevenLabs adapter (premium streaming TTS)
- [ ] Local whisper.cpp adapter (offline STT)
- [ ] System TTS fallback (macOS `say`, Linux `espeak`)
- [ ] `rusvel talk` CLI command with microphone capture
- [ ] Wake word detection (keyword spotting via local VAD)
- [ ] `POST /api/voice/transcribe` — upload audio, get text
- [ ] `POST /api/voice/synthesize` — text to audio
- [ ] `WS /api/voice/stream` — real-time voice conversation
- [ ] Auto-transcribe voice messages from channels
- [ ] Frontend: Voice button in chat, audio player for TTS responses

---

## 8. Canvas & A2UI Protocol

### What OpenClaw Has

**A2UI (Agent 2 UI)** — agents push visual content to connected devices:

```
Canvas Host (HTTP, port 18793)
    ↓ WebSocket bridge
Connected nodes (macOS/iOS/Android)
```

**Canvas actions:**
- `present(targetUrl)` — show HTML content
- `hide()` — hide canvas
- `navigate(url)` — change content
- `eval(js)` — execute JavaScript in canvas context
- `snapshot()` — screenshot current state
- `reset()` — clear canvas

**Canvas content:**
- Full HTML/CSS/JS rendering
- Live reload (watch directory, inject WebSocket client)
- Agent-generated dashboards, charts, forms
- Interactive: user input flows back to agent

**Binding modes:**
- Loopback (local only)
- LAN (local network)
- Tailnet (Tailscale)
- Auto (best available)

### What We Build

```rust
// Agent-side canvas actions
pub enum CanvasAction {
    Push(CanvasContent),
    Update { element_id: String, props: serde_json::Value },
    Reset,
    Eval { script: String },
    Snapshot { format: ImageFormat },
    Hide,
}

pub enum CanvasContent {
    Html(String),
    Markdown(String),
    Chart { chart_type: ChartType, data: serde_json::Value, options: serde_json::Value },
    Table { headers: Vec<String>, rows: Vec<Vec<serde_json::Value>> },
    Form { schema: serde_json::Value, on_submit: String },  // tool call ID
    Kanban { columns: Vec<KanbanColumn> },
    Timeline { events: Vec<TimelineEvent> },
    Diff { before: String, after: String },
    Custom { component: String, props: serde_json::Value },
}

// Canvas tool (registered in tool registry)
pub struct CanvasTool;
// Parameters: action: "push"|"update"|"reset"|"eval"|"snapshot"
// + content payload

// Department-specific canvas presets
// Forge: mission dashboard, agent activity monitor
// Harvest: pipeline kanban, opportunity scoring cards
// Content: calendar, draft preview, analytics
// Finance: P&L table, runway chart, cashflow
// GTM: deal board, outreach timeline
// Flow: live workflow execution graph
// Code: dependency graph, complexity heatmap
```

**Integration with existing AgentEvent:**

```rust
pub enum AgentEvent {
    TextDelta { text: String },
    ToolCall { tool_call_id: String, name: String, args: serde_json::Value },
    ToolResult { tool_call_id: String, name: String, output: String, is_error: bool },
    StateDelta { delta: serde_json::Value },
    CanvasPush { content: CanvasContent },   // NEW
    CanvasAction { action: CanvasAction },     // NEW
    Done { output: AgentOutput },
    Error { message: String },
}
```

### Deliverables

- [ ] CanvasAction + CanvasContent types in rusvel-core
- [ ] AgentEvent::CanvasPush / CanvasAction variants
- [ ] `canvas_push`, `canvas_update`, `canvas_reset`, `canvas_snapshot` tools
- [ ] Frontend: `<Canvas>` component that renders CanvasContent variants
- [ ] Chart rendering (reuse existing LayerChart/D3)
- [ ] Table rendering with sort/filter
- [ ] Form rendering with schema → input fields → tool callback
- [ ] Kanban board rendering
- [ ] Live reload via SSE
- [ ] Canvas panel in department chat (split view)
- [ ] Department canvas presets (per-department default views)

---

## 9. Cron, Webhooks & Automation Triggers

### What OpenClaw Has

**Cron system:**
- Agent-accessible cron tool (create/update/delete/list)
- Cron expression parsing
- Delivery targeting (which channel to announce in)
- Retry policy: maxAttempts, backoffMs, retryOn filters
- Failure alerting: after N failures, cooldown, announce/webhook modes
- Run log rotation: maxBytes, keepLines
- Concurrent run limiting
- Session retention pruning (24h default)

**Webhook system:**
- External webhook serving with token auth
- Session key allowlisting
- Agent ID routing restrictions
- Gmail Pub/Sub integration (inbound email → agent)
- Tailscale Serve/Funnel for public endpoints
- SSRF guard on outbound webhook calls

**Hook mappings:**
```typescript
hookMappings: [
  { match: { path: "/github/push" }, action: "agent", channel: "slack" },
  { match: { source: "stripe" }, action: "wake", wakeMode: "once" },
]
```

### What RUSVEL Has Today

Job queue with `ScheduledCron` job kind + hook dispatch with 3 action types. Missing:
- No cron expression parsing (just scheduled_at timestamps)
- No retry policies on cron jobs
- No failure alerting
- No run log rotation
- No concurrent run limits
- No inbound webhook endpoints
- No SSRF protection on outbound hooks
- No Gmail/email triggers
- No hook mappings (path → action routing)

### What We Build

```rust
// Cron enhancement
pub struct CronJob {
    pub id: String,
    pub name: String,
    pub schedule: CronSchedule,            // "0 9 * * MON-FRI"
    pub department: String,
    pub prompt: String,                     // what the agent should do
    pub delivery: Option<DeliveryTarget>,   // channel + recipient for results
    pub retry: RetryPolicy,
    pub failure_alert: Option<FailureAlert>,
    pub max_concurrent: u32,
    pub enabled: bool,
    pub metadata: serde_json::Value,
}

pub struct RetryPolicy {
    pub max_attempts: u32,
    pub backoff_ms: u64,
    pub retry_on: Vec<RetryTrigger>,       // RateLimit, Timeout, ServerError, Network
}

pub struct FailureAlert {
    pub after_failures: u32,
    pub cooldown: Duration,
    pub mode: AlertMode,                    // Channel, Webhook, Both
    pub webhook_url: Option<String>,
    pub channel: Option<String>,
}

// Inbound webhooks
pub struct WebhookEndpoint {
    pub id: String,
    pub path: String,                       // "/webhooks/github/push"
    pub secret: Option<String>,             // HMAC-SHA256 verification
    pub routing: WebhookRouting,
    pub enabled: bool,
}

pub enum WebhookRouting {
    Event { event_kind: String },           // Emit as event, let hooks handle
    Department { dept: String, prompt_template: String },
    Agent { prompt_template: String },       // Direct to forge god agent
    HookMapping(Vec<HookMapping>),          // Path/source → action routing
}

pub struct HookMapping {
    pub match_pattern: MatchPattern,        // path, source, content regex
    pub action: HookMappingAction,          // Agent, Wake, Ignore
    pub department: Option<String>,
    pub channel: Option<String>,            // deliver result to channel
}
```

### Deliverables

- [ ] Cron expression parser (use `cron` crate)
- [ ] CronJob with retry policies, failure alerting, concurrent limits
- [ ] Run log persistence with rotation
- [ ] `rusvel cron create/list/delete/run` CLI commands
- [ ] Cron tool for agents (schedule tasks from conversations)
- [ ] Inbound webhook endpoints: `POST /api/webhooks/{path}`
- [ ] HMAC-SHA256 signature verification
- [ ] SSRF guard on outbound webhook/HTTP calls
- [ ] WebhookRouting: event emission, department routing, agent dispatch
- [ ] HookMapping: path/source pattern → action routing
- [ ] Gmail Pub/Sub adapter (inbound email → agent) — future
- [ ] Frontend: Cron job editor, webhook endpoint config, run log viewer

---

## 10. Skills Platform & ClawHub Marketplace

### What OpenClaw Has

**51 bundled skills** + community skills via ClawHub:

```
skills/
├── 1password/          — password manager integration
├── apple-notes/        — Apple Notes access
├── apple-reminders/    — Apple Reminders access
├── bear-notes/         — Bear note-taking app
├── canvas/             — A2UI canvas control
├── clawhub/            — skill marketplace search
├── coding-agent/       — code editing agent
├── discord/            — Discord-specific actions
├── gh-issues/          — GitHub issues management
├── github/             — GitHub operations
├── himalaya/           — email client integration
├── model-usage/        — LLM usage tracking
├── peekaboo/           — macOS automation
├── session-logs/       — session log export
├── skill-creator/      — create new skills
├── slack/              — Slack-specific actions
├── tmux/               — terminal multiplexer
├── voice-call/         — telephony
├── weixin/             — WeChat integration
└── ... (51 total)
```

**Each skill has:**
- `SKILL.md` — metadata, description, config schema, requirements
- Implementation files (TypeScript)
- Dependencies declaration
- Install gating (check requirements before enabling)

**Skill distribution tiers:**
- **Bundled** — ships with core
- **Managed** — maintained by OpenClaw team, installed separately
- **Workspace** — user-created, per-workspace directory
- **Community** — ClawHub marketplace

**Status UI:**
- Tabs: All / Ready / Needs Setup / Disabled
- Click-to-detail with API key entry, install actions
- Requirements checking + dependency installation

### What RUSVEL Has Today

Skills as ObjectStore entries with `{{input}}` interpolation. Very basic:
- No requirements/dependencies
- No install gating
- No marketplace
- No categories/tabs
- No skill creation tool

### What We Build

```rust
pub struct SkillDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: Option<String>,
    pub prompt_template: String,
    pub requirements: Vec<SkillRequirement>,
    pub config_schema: Option<serde_json::Value>,
    pub department: Option<String>,           // restrict to department
    pub tier: SkillTier,
    pub status: SkillStatus,
    pub tags: Vec<String>,
    pub metadata: serde_json::Value,
}

pub enum SkillTier {
    Bundled,           // ships with rusvel
    Workspace,         // user-created in workspace
    Community,         // installed from marketplace
}

pub enum SkillStatus {
    Ready,
    NeedsSetup,        // missing config/credentials
    Disabled,
    Error(String),
}

pub struct SkillRequirement {
    pub kind: RequirementKind,
    pub name: String,
    pub check: String,  // shell command to verify
}

pub enum RequirementKind {
    Credential(String),    // e.g., "OPENAI_API_KEY"
    Binary(String),        // e.g., "ffmpeg"
    Channel(String),       // e.g., "slack" (requires channel connected)
    Port(String),          // e.g., "voice" (requires VoicePort)
}

// Skill creator tool (like OpenClaw's skill-creator)
// `!skill create <description>` → generates SKILL.md + implementation
```

**Bundled skills for RUSVEL:**

| Skill | Department | Description |
|-------|-----------|-------------|
| `daily-briefing` | Forge | Morning summary from all departments |
| `invoice-draft` | Finance | Generate invoice from deal data |
| `proposal-writer` | Harvest | Write proposal from opportunity |
| `code-review` | Code | Review PR with analysis |
| `content-repurpose` | Content | Turn blog → tweets → LinkedIn |
| `outreach-sequence` | GTM | Multi-step outreach campaign |
| `standup-report` | Forge | Generate standup from recent events |
| `competitor-scan` | Harvest | Analyze competitor URLs |
| `seo-audit` | Distro | Analyze page SEO |
| `contract-review` | Legal | Review contract terms |

### Deliverables

- [ ] Enhanced SkillDefinition with requirements, status, tiers, config schema
- [ ] Skill requirements checking (credentials, binaries, channels)
- [ ] Skill status: Ready / NeedsSetup / Disabled / Error
- [ ] `!skill create <description>` — AI-generates skill from description
- [ ] `rusvel skill list/install/enable/disable` CLI commands
- [ ] 10 bundled skills (see table above)
- [ ] Workspace skills directory (`~/.rusvel/skills/`)
- [ ] Skill config UI with setup wizard
- [ ] Frontend: Skills page with tabs (All/Ready/NeedsSetup/Disabled)
- [ ] Frontend: Skill detail dialog with config entry, install action
- [ ] Future: Community marketplace API

---

## 11. Provider Failover & Auth Profiles

### What OpenClaw Has

**Multi-provider failover with auth profile rotation:**

```typescript
// Multiple auth profiles per provider
authProfiles: [
  { id: "claude-primary", provider: "anthropic", model: "claude-opus-4" },
  { id: "claude-backup", provider: "anthropic", model: "claude-sonnet-4" },
  { id: "openai-fallback", provider: "openai", model: "gpt-4.1" },
]

// Automatic failover triggers:
retryOn: ["rate_limit", "overloaded", "network", "timeout", "server_error"]

// Per-provider backoff:
backoffMs: [1000, 2000, 4000, 8000, 16000, 32000, 60000]
```

**Model catalog system:**
- Async model catalog loading with backoff retry (1s to 60s)
- Model capability detection (1M context, vision, extended thinking)
- Config-driven context window overrides
- Provider-specific model ID normalization

**Cost tracking:**
- Per-session: inputTokens, outputTokens, totalTokens, estimatedCostUsd
- Compaction tracking: compactionCount, memoryFlushAt

### What RUSVEL Has Today

`rusvel-llm` with 4 hardcoded providers + ModelTier routing + CostTracker. Missing:
- No automatic failover between providers
- No auth profile rotation
- No backoff retry with exponential delay
- No model catalog with capability detection
- CostTracker exists but not exposed in sessions

### What We Build

```rust
pub struct ProviderFailover {
    pub profiles: Vec<ProviderProfile>,
    pub retry_on: Vec<FailoverTrigger>,
    pub max_retries: u32,
    pub backoff: BackoffStrategy,
}

pub struct ProviderProfile {
    pub id: String,
    pub provider: String,
    pub model: String,
    pub priority: u32,
    pub credential: CredentialHandle,
    pub rate_limit_state: Arc<RwLock<RateLimitState>>,
}

pub enum FailoverTrigger {
    RateLimit,
    Overloaded,
    Timeout,
    ServerError,
    NetworkError,
}

pub struct BackoffStrategy {
    pub initial_ms: u64,
    pub max_ms: u64,
    pub multiplier: f64,
    pub jitter: f64,
}

// Model catalog
pub struct ModelCatalog {
    pub models: HashMap<String, ModelCapabilities>,
}

pub struct ModelCapabilities {
    pub context_window: usize,
    pub supports_vision: bool,
    pub supports_extended_thinking: bool,
    pub supports_streaming: bool,
    pub supports_tool_use: bool,
    pub cost_per_input_token: f64,
    pub cost_per_output_token: f64,
}
```

### Deliverables

- [ ] ProviderFailover engine with automatic retry + rotation
- [ ] Auth profile system (multiple credentials per provider)
- [ ] Exponential backoff with jitter
- [ ] Rate limit state tracking per profile
- [ ] Model catalog with capability detection
- [ ] Context window overrides per model
- [ ] Per-session cost accumulation exposed in Session model
- [ ] `rusvel config provider add/remove/list` CLI commands
- [ ] Frontend: Provider config with failover chain editor

---

## 12. Browser Automation Enhancements

### What OpenClaw Has

**Dedicated browser control** beyond basic CDP:

```
browser-tool.ts:
  ├── executeActAction()       — DOM manipulation (click, type, select)
  ├── executeSnapshotAction()  — Screenshots + page snapshots
  ├── executeConsoleAction()   — JS eval in browser context
  └── executeTabsAction()      — Tab management (open, close, switch)

Features:
  - DOM querying & node selection (CSS selectors)
  - File uploads/downloads
  - Dialog/file chooser handling
  - Profile management (user agents, cookies)
  - Proxy support (SSRF-aware)
  - Screenshot/PDF export
  - Accessibility tree snapshots
```

### What RUSVEL Has Today

`rusvel-cdp` with basic BrowserPort: connect, tabs, observe, evaluate_js, navigate. Missing:
- No DOM manipulation actions (click, type, select)
- No file upload/download
- No dialog handling
- No profile management
- No accessibility tree
- No screenshot tool for agents

### What We Build

```rust
// Enhanced BrowserPort actions
pub enum BrowserAction {
    // Navigation
    Navigate { url: String },
    Back,
    Forward,
    Reload,

    // DOM interaction
    Click { selector: String },
    Type { selector: String, text: String },
    Select { selector: String, value: String },
    ScrollTo { selector: Option<String>, x: i32, y: i32 },
    WaitFor { selector: String, timeout_ms: u64 },

    // Capture
    Screenshot { full_page: bool, selector: Option<String> },
    Pdf { landscape: bool },
    Snapshot,                       // DOM + accessibility tree
    ConsoleEval { script: String },

    // Tab management
    NewTab { url: Option<String> },
    CloseTab { tab_id: String },
    SwitchTab { tab_id: String },
    ListTabs,

    // File handling
    Upload { selector: String, file_path: String },
    DownloadWait { timeout_ms: u64 },

    // Profile
    SetCookies(Vec<Cookie>),
    ClearCookies,
    SetUserAgent(String),
}
```

### Deliverables

- [ ] Enhanced BrowserPort with DOM actions (click, type, select, scroll)
- [ ] Screenshot tool for agents (full page + element targeting)
- [ ] Accessibility tree snapshot (for agent understanding)
- [ ] File upload/download handling
- [ ] Dialog interception (alert, confirm, prompt, file chooser)
- [ ] Cookie/profile management
- [ ] PDF export
- [ ] WaitFor selector with timeout
- [ ] `browser_act`, `browser_snapshot`, `browser_console` agent tools

---

## 13. Native Companion Apps

### What OpenClaw Has

**Three native apps:**

| Platform | Stack | Features |
|----------|-------|----------|
| macOS | Swift 6.2, SwiftUI, Observation | Menu bar, Voice Wake, PTT, Canvas WebView, remote gateway |
| iOS | Swift 6.2, SwiftUI | Canvas, Voice Wake, Talk Mode, camera, screen recording, Bonjour |
| Android | Kotlin, Jetpack Compose | Chat, Voice, Canvas, camera, screen recording, device commands |

**Shared capabilities:**
- Gateway WebSocket connection
- Canvas WebView rendering
- Voice capture → STT → agent → TTS → playback
- Camera/screen capture → agent vision
- Device sensors: location, motion, calendar, contacts

### What We Build (Phase 5+)

This is a future vision, but the architecture should support it now:

```rust
// Node connection protocol (devices register with gateway)
pub struct NodeConnection {
    pub device_id: String,
    pub device_type: DeviceType,  // MacOS, IOS, Android, Browser
    pub capabilities: Vec<NodeCapability>,
    pub connected_at: DateTime<Utc>,
}

pub enum NodeCapability {
    Camera,
    Microphone,
    Screen,
    Location,
    Notifications,
    Canvas,
    FileSystem,
}

// Node commands (gateway → device)
pub enum NodeCommand {
    CameraSnap,
    CameraClip { duration: Duration },
    ScreenRecord { duration: Duration },
    LocationGet,
    NotificationSend { title: String, body: String },
    CanvasPush(CanvasContent),
}
```

### Deliverables (Architecture Only for Now)

- [ ] NodeConnection + NodeCapability types in rusvel-core
- [ ] NodeCommand enum for device control
- [ ] WebSocket endpoint for device connections: `WS /api/nodes/connect`
- [ ] Node registration + capability discovery
- [ ] Future: macOS companion app (Swift + SwiftUI)
- [ ] Future: iOS companion app
- [ ] Future: Android companion app

---

## 14. Onboarding, Doctor & Diagnostics

### What OpenClaw Has

**`openclaw onboard`** — interactive setup wizard:
- Step-by-step gateway config
- Channel pairing walkthroughs
- API key entry with validation
- Skill discovery and setup
- Daemon installation (launchd/systemd)

**`openclaw doctor`** — system health check:
- Config validation
- Channel connection status
- DM policy security audit
- Missing dependency detection
- Port conflict detection
- Migration recommendations

**Plugin diagnostics:**
- Per-plugin: status, enabled, error, tool/hook/channel/provider counts
- Error tracking with configurable severity

### What RUSVEL Has Today

No onboarding wizard. No doctor command. No diagnostics beyond logs.

### What We Build

```bash
# Onboarding
rusvel onboard                    # Interactive setup wizard
  → Step 1: Configure LLM provider (Ollama/Claude/OpenAI)
  → Step 2: Test LLM connection
  → Step 3: Create first session
  → Step 4: Connect channels (optional)
  → Step 5: Install daemon (optional)
  → Step 6: Open web UI

# Doctor
rusvel doctor                     # System health check
  ✓ Database: SQLite WAL mode, 45 tables, 2.3MB
  ✓ LLM: Ollama connected (llama3.2 available)
  ✗ Claude API: ANTHROPIC_API_KEY not set
  ✓ Channels: 0 connected (run `rusvel channel connect` to add)
  ✓ Jobs: Queue empty, worker running
  ✓ Memory: FTS5 index healthy, 142 entries
  ✗ Vector: LanceDB not initialized (run `rusvel embed init`)
  ✓ Frontend: Build embedded (2.1MB)
  ⚠ Skills: 3 need setup (run `rusvel skill list --needs-setup`)
  ✓ Ports: 3000 available

# Diagnostics
rusvel diag                       # Detailed system report
rusvel diag --json                # Machine-readable
```

### Deliverables

- [ ] `rusvel onboard` interactive CLI wizard
- [ ] `rusvel doctor` health check (DB, LLM, channels, jobs, memory, vector, frontend)
- [ ] Per-department diagnostics (tool count, event count, hook count)
- [ ] Provider health probes (ping LLM endpoints)
- [ ] Channel connection status
- [ ] Missing credential detection
- [ ] `POST /api/system/doctor` — health check API
- [ ] Frontend: System status dashboard

---

## 15. Deployment: Docker, Fly.io, Daemon Install

### What OpenClaw Has

**Docker:**
- Multi-stage Dockerfile (ext-deps → build → runtime)
- Bookworm/bookworm-slim variants
- Extension opt-in via build args
- docker-compose for gateway + CLI services
- Sandbox support (nested Docker)
- Podman alternative support

**Fly.io:**
- `fly.toml` with shared-cpu-2x, 2GB RAM
- Persistent volume at `/data`
- force_https, auto_stop_machines=false
- Single-region deployment

**Daemon install:**
- macOS: launchd plist
- Linux: systemd user service
- Auto-start on boot
- Health monitoring

### What RUSVEL Has Today

`cargo run` and nothing else. No Docker, no deployment, no daemon.

### What We Build

```dockerfile
# Dockerfile
FROM rust:1.85 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release
RUN cd frontend && pnpm install && pnpm build

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/rusvel /usr/local/bin/
COPY --from=builder /app/frontend/build /app/frontend/build
EXPOSE 3000
CMD ["rusvel", "--bind", "0.0.0.0:3000"]
```

```yaml
# docker-compose.yml
services:
  rusvel:
    build: .
    ports: ["3000:3000"]
    volumes:
      - rusvel-data:/data
    environment:
      - RUSVEL_DB_PATH=/data/rusvel.db
      - ANTHROPIC_API_KEY=${ANTHROPIC_API_KEY}
    restart: unless-stopped
```

```toml
# fly.toml
app = "rusvel"
[build]
dockerfile = "Dockerfile"
[http_service]
internal_port = 3000
force_https = true
[mounts]
source = "rusvel_data"
destination = "/data"
[[vm]]
size = "shared-cpu-2x"
memory = "2gb"
```

### Deliverables

- [ ] Dockerfile (multi-stage: Rust build + frontend build → slim runtime)
- [ ] docker-compose.yml (gateway + volume + env)
- [ ] fly.toml for Fly.io deployment
- [ ] `rusvel install-daemon` (macOS launchd / Linux systemd)
- [ ] `rusvel uninstall-daemon`
- [ ] Health endpoint: `GET /api/health`
- [ ] Graceful shutdown handling
- [ ] Persistent data directory configuration
- [ ] `scripts/deploy.sh` (Docker build + push)

---

## 16. Implementation Roadmap

### Priority Matrix

| # | Workstream | Impact | Effort | Dependencies | Phase |
|---|-----------|--------|--------|-------------|-------|
| **0** | **Wire 10 unwired departments** | **Critical** | **Medium** | **None** | **NOW** |
| **0b** | **Persist job queue to SQLite** | **Critical** | **Low** | **None** | **NOW** |
| **0c** | **5-zone icon rail restructuring** | **High UX** | **Medium** | **None** | **NOW** |
| 1 | Plugin SDK | Foundation | Medium | Dept wiring done | 2 |
| 2 | Hook System (20+ events) | High | Medium | Plugin SDK | 2 |
| 3 | Provider Failover | High | Low | None | 2 |
| 4 | Context Pruning + Memory | High | Medium | None | 2 |
| 5 | Agent Runtime (subagents, forking) | High | High | Hooks | 2-3 |
| 6 | Channel Infrastructure | Transformative | High | Plugin SDK, Hooks, Auth | 3 |
| 7 | Message Routing + DM Security | High | Medium | Channels | 3 |
| 8 | Cron + Webhooks + Automation | High | Medium | Hooks, Auth middleware | 3 |
| 9 | Skills Platform | Medium | Medium | Plugin SDK | 3 |
| 10 | Browser Enhancements | Medium | Low | None | 3 |
| 11 | Voice (STT/TTS/Talk) | Medium | High | Channels | 4 |
| 12 | Canvas A2UI | Medium | Medium | 5-zone layout done | 4 |
| 13 | Onboarding + Doctor | High UX | Low | All | 3 |
| 14 | Deployment (Docker/Fly/Daemon) | High ops | Low | Auth middleware | 3 |
| 15 | Native Apps | Moon shot | Very High | Canvas, Voice, Channels | 5 |

### Sequencing

```
PHASE 0 (NOW) — Close the Wiring Gap
├── Wire 45 tools across 10 dept-* crates
├── Wire event handlers + job handlers + approval gates
├── Persist job queue to SQLite (replace Vec<Job>)
├── Add auth middleware on API routes
├── 5-zone icon rail restructuring (icon rail + section sidebar + context panel + bottom panel)
└── Proves: all 13 departments agent-accessible, frontend bookmarkable

PHASE 2 — Agent Intelligence
├── Provider Failover + Auth Profiles
├── Context Pruning + Memory Enhancements
├── Plugin SDK (generalize DepartmentApp → PluginEntry)
├── Hook System (20+ lifecycle events)
└── Agent Runtime (subagents, forking, thinking levels)

PHASE 3 — GoToMarket + Operations
├── Channel Infrastructure (Email, Slack, Telegram)
│   └── Frontend: /inbox (icon rail), /dept/{id}/channels (section sidebar)
├── Message Routing + DM Security + Pairing
├── Cron + Webhooks + Automation Triggers
├── Skills Platform + Bundled Skills
│   └── Frontend: /dept/{id}/skills enhanced (tabs: All/Ready/NeedsSetup/Disabled)
├── Browser Automation Enhancements
├── Onboarding + Doctor + Diagnostics
│   └── Frontend: /settings/doctor, `rusvel onboard` CLI
└── Deployment (Docker, Fly.io, Daemon)

PHASE 4 — Cross-Engine Intelligence
├── Voice (STT, TTS, Wake Word, Talk Mode)
│   └── Frontend: voice button in chat, /voice (icon rail)
├── Canvas A2UI Protocol
│   └── Frontend: Canvas panel in /dept/{id}/chat (split view)
├── More Channel Adapters (Discord, WhatsApp, SMS)
└── Cross-channel Unified Inbox
    └── Frontend: /inbox (icon rail) with channel indicators

PHASE 5 — Ecosystem
├── Dynamic Plugin Loading (.so/.dylib)
├── Community Skill Marketplace
├── Native Companion Apps (macOS, iOS, Android)
└── Node Protocol (device capabilities)
```

### Crate Impact Summary

#### Phase 0 (NOW) — Close the Wiring Gap

| Crate | Action | Est. Lines |
|-------|--------|-----------|
| `dept-harvest` | Wire 5 tools + event handlers | +120 |
| `dept-flow` | Wire 5 tools | +100 |
| `dept-gtm` | Wire 5 tools + OutreachSend job handler | +130 |
| `dept-finance` | Wire 5 tools | +100 |
| `dept-product` | Wire 4 tools | +80 |
| `dept-growth` | Wire 4 tools | +80 |
| `dept-distro` | Wire 4 tools | +80 |
| `dept-legal` | Wire 4 tools | +80 |
| `dept-support` | Wire 5 tools | +100 |
| `dept-infra` | Wire 4 tools | +80 |
| `rusvel-jobs` | Persist to SQLite (replace Vec<Job>) | +150 |
| `rusvel-api` | Auth middleware (basic token/session) | +100 |
| `frontend/` | 5-zone icon rail restructuring (17 files) | +800 |
| **Phase 0 total** | | **~2000** |

#### Phases 2-5 — OpenClaw Integration

| Crate | Action | Est. Lines |
|-------|--------|-----------|
| `rusvel-plugin-sdk` | **New** — plugin system | ~600 |
| `rusvel-channel` | **New** — channel adapters + routing | ~1200 |
| `rusvel-voice` | **New** — STT/TTS providers | ~800 |
| `rusvel-webhook` | **New** — inbound webhook handling | ~500 |
| `rusvel-cron` | **New** — cron expression + scheduling | ~400 |
| `dept-messaging` | **New** — messaging department | ~400 |
| `rusvel-core` | Modify — ChannelPort, VoicePort, Canvas types, Hook events | +400 |
| `rusvel-agent` | Modify — failover, subagents, forking, pruning, thinking | +600 |
| `rusvel-llm` | Modify — provider failover, auth profiles, model catalog | +400 |
| `rusvel-tool` | Modify — browser tools, canvas tools, cron tools | +300 |
| `rusvel-memory` | Modify — citations, flush planning, retention | +200 |
| `rusvel-api` | Modify — channels, webhooks, voice, inbox, doctor routes | +800 |
| `rusvel-cli` | Modify — onboard, doctor, pairing, talk, cron commands | +500 |
| `rusvel-app` | Modify — wire new ports, boot plugins, daemon install | +200 |
| `frontend/` | Modify — inbox, canvas, skills, channels, voice, onboarding | +3000 |
| **Phases 2-5 total** | 6 new crates + modifications | **~10,300** |

**Grand total:** ~12,300 new lines (Rust + frontend). Workspace 50 → 56 crates.

Architecture holds — Phase 0 is pure wiring (no new crates), all later growth is adapters + one SDK crate.

---

## 17. Frontend Compliance Matrix (5-Zone Icon Rail Layout)

Every new UI surface must comply with `docs/design/ui-redesign-final.md`. Here's where each feature lands:

### Icon Rail (48px, top section — global pages)

| Icon | URL | Workstream | Description |
|------|-----|-----------|-------------|
| **Inbox** | `/inbox` | Channels (#2) | Unified multi-channel inbox |
| **Voice** | `/voice` | Voice (#7) | Voice conversation interface |
| (Settings sub) | `/settings/doctor` | Onboarding (#14) | System health dashboard |

### Section Sidebar (200px, manifest-driven — dept pages)

| Section | URL Pattern | Workstream | Description |
|---------|------------|-----------|-------------|
| **Channels** | `/dept/{id}/channels` | Channels (#2) | Per-dept channel config + routing rules |
| **Cron** | `/dept/{id}/cron` | Automation (#9) | Department cron jobs + run logs |
| **Skills** (enhanced) | `/dept/{id}/skills` | Skills (#10) | Tabs: All/Ready/NeedsSetup/Disabled |

### Main Content (flexible)

| Page | URL | Workstream | Enhancement |
|------|-----|-----------|------------|
| **Chat** | `/dept/{id}/chat` | Canvas (#8), Voice (#7) | Canvas renders inline, voice button in input |
| **Settings** | `/dept/{id}/settings` | Provider Failover (#11) | Failover chain editor |
| **Flows** | `/flows` | Cron (#9) | Webhook trigger nodes |
| **Agents** | `/dept/{id}/agents` | Agent Runtime (#5) | Subagent spawn config |

### Context Panel (320px, collapsible right)

| Mode | Workstream | Content |
|------|-----------|---------|
| **Quick Chat** | All | DepartmentChat in compact mode — chat alongside any section |
| **Canvas** | Canvas (#8) | A2UI canvas content pushed by agent |
| **Properties** | All | Selected item detail (agent config, skill params, etc.) |
| **Execution** | All | Tool calls, approvals, job output during agent runs |

### Bottom Panel (collapsible)

| Tab | Workstream | Content |
|-----|-----------|---------|
| **Terminal** | Existing | PTY output via TerminalPort WebSocket |
| **Jobs** | Cron (#9) | Job execution status from JobPort |
| **Events** | Hook System (#4) | Live event stream with hook diagnostics |

### Component Reuse

| Existing Component | Used By | Change |
|-------------------|---------|--------|
| `DepartmentChat` | Context panel (compact mode) | Add compact prop for 320px width |
| `SkillsTab` | Skills Platform | Add status tabs, requirements UI |
| `EventsTab` | Bottom panel events tab | Reuse for live event stream |
| `WorkflowBuilder` | Cron, Webhooks | Add webhook trigger node type |
| `EngineTab` | All 10 newly-wired depts | No change — tools appear automatically |
| `ActionsTab` | All 10 newly-wired depts | Quick actions from manifest already correct |
| `DeptTerminal` | Bottom panel terminal tab | Reuse as-is |

---

## Reference

| Source | Location |
|--------|----------|
| OpenClaw repo | `openclaw/openclaw` (TypeScript, MIT, 296K objects) |
| OpenClaw docs | https://docs.openclaw.ai |
| Plugin SDK | `openclaw/src/plugin-sdk/` + `openclaw/extensions/` (84 extensions) |
| Channel types | `openclaw/src/channels/plugins/types.plugin.ts` |
| Hook system | `openclaw/src/plugins/hooks.ts` (20+ events) |
| Agent runtime | `openclaw/src/agents/pi-embedded-runner/run/attempt.ts` (73KB) |
| Context pruning | `openclaw/src/agents/pi-extensions/context-pruning/pruner.ts` |
| Canvas A2UI | `openclaw/src/canvas-host/a2ui.ts` |
| Skills | `openclaw/skills/` (51 bundled) |
| Cron | `openclaw/src/gateway/server-cron.ts` |
| Memory | `openclaw/packages/memory-host-sdk/` |
| Native apps | `openclaw/apps/` (macOS, iOS, Android) |
| Config types | `openclaw/src/config/types.*.ts` |
| RUSVEL roadmap | `docs/plans/roadmap-v2.md` |
| A2UI vision | `docs/plans/a2ui-department-apps.md` |
| RUSVEL ADRs | `docs/design/decisions.md` (14 ADRs) |
