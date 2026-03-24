# Proposal: OpenClaw Capabilities → RUSVEL Integration

**Date:** 2026-03-24
**Status:** Draft
**Scope:** What OpenClaw features can strengthen RUSVEL, how to integrate them, and what to skip

---

## Executive Summary

OpenClaw is a **multi-channel AI gateway** — it connects one AI to 20+ messaging platforms (WhatsApp, Telegram, Slack, Discord, Signal, iMessage, etc.) with voice, browser control, device pairing, and a live canvas. RUSVEL is a **solo founder operating system** — 12 departments, hexagonal architecture, single binary.

The overlap is small but the synergy is massive: RUSVEL has deep domain engines but no external reach. OpenClaw has broad external reach but no domain intelligence. Integrating select OpenClaw capabilities turns RUSVEL from a dashboard you visit into an **always-on assistant that meets you where you already are**.

---

## Feature-by-Feature Analysis

### 1. MULTI-CHANNEL MESSAGING GATEWAY

**What OpenClaw has:** 20+ channel plugins (WhatsApp, Telegram, Slack, Discord, Signal, Matrix, IRC, LINE, Teams, etc.) with unified message routing, session threading, group activation, and reply-back.

**What it adds to RUSVEL:**

| Use Case | Department | Example |
|----------|-----------|---------|
| Client comms from CRM | GTM | "Draft reply to John's WhatsApp about the proposal" |
| Support tickets from DMs | Support | Auto-triage incoming Telegram messages as tickets |
| Content publishing notifications | Content | "Your blog post just went live" → Slack + Telegram |
| Opportunity alerts | Harvest | New Upwork match → WhatsApp ping with score |
| Finance alerts | Finance | "Invoice #42 overdue by 7 days" → SMS/Signal |
| Daily mission delivery | Forge | Morning plan pushed to your preferred channel |
| Approval requests | All | "Approve outreach to @handle?" → tap reply in Telegram |

**Recommendation: YES — High priority**

**How to integrate:**
- Add a `rusvel-channels` adapter crate implementing a new `ChannelPort` trait
- Each channel is a plugin (reuse OpenClaw's TypeScript plugins via subprocess or rewrite top 3-5 in Rust)
- Route inbound messages through the existing department chat system
- Route outbound notifications through the event system (hooks already exist, just need channel targets)

**Tradeoffs:**
- (+) Turns RUSVEL into a conversational interface, not just a web dashboard
- (+) Approval workflows become instant (reply from phone)
- (+) Content/Harvest/Support engines get real distribution channels
- (−) TypeScript dependency if reusing OpenClaw plugins directly (breaks single-binary promise)
- (−) Each channel has its own auth dance (WhatsApp QR scan, Telegram bot token, Slack OAuth)
- (−) Maintenance burden: messaging APIs change frequently

**Mitigation:** Start with 3 channels max (Telegram, Slack, WhatsApp). Use subprocess bridge to OpenClaw's Node.js plugins rather than rewriting. Keep the `ChannelPort` trait in rusvel-core so future Rust-native plugins can replace them.

---

### 2. VOICE INTERACTION (Wake Word + Talk Mode + TTS)

**What OpenClaw has:** Wake word detection (macOS/iOS), continuous voice mode (Android), ElevenLabs/Edge TTS, speech-to-text pipeline.

**What it adds to RUSVEL:**
- Hands-free mission briefing: "Hey Rusvel, what's my plan today?"
- Voice-driven CRM updates: "Log a call with Sarah, discussed pricing, she's interested"
- Quick content dictation: "Draft a tweet about our new Rust workshop"
- Finance entries: "Add expense, $47 lunch with client"

**Recommendation: NO — Skip for now**

**Reasoning:**
- Voice is a UX layer, not a capability gap. RUSVEL's bottleneck is wiring its existing engines, not adding new input modalities.
- Wake word detection requires native platform code (Swift/Kotlin) — antithetical to single-binary Rust.
- The REPL and TUI already provide fast interaction for a solo founder at their desk.
- If needed later, better to integrate with system-level voice assistants (Siri Shortcuts, Android Intents) than building a custom voice stack.

---

### 3. BROWSER AUTOMATION (CDP Control)

**What OpenClaw has:** Dedicated Chromium instance under CDP (Chrome DevTools Protocol) control — navigate, screenshot, fill forms, extract data, persist profiles.

**What it adds to RUSVEL:**

| Use Case | Department | Example |
|----------|-----------|---------|
| Opportunity scraping | Harvest | Navigate Upwork/LinkedIn, extract gig details behind login walls |
| Content publishing | Content | Auto-post to platforms without APIs (Substack, Medium) |
| Competitor monitoring | Growth | Screenshot competitor pricing pages, detect changes |
| Invoice generation | Finance | Fill invoice templates, download PDFs |
| SEO auditing | Distribution | Run Lighthouse, extract meta tags, check indexing |
| Client demos | GTM | Automated walkthroughs with screenshots |

**Recommendation: YES — Medium priority**

**How to integrate:**
- Add a `rusvel-browser` adapter crate wrapping `chromiumoxide` (Rust CDP library) or `headless_chrome`
- Expose as a tool in the tool registry: `browser.navigate`, `browser.screenshot`, `browser.click`, `browser.extract`
- Harvest engine uses it for login-walled scraping
- Content engine uses it for platforms without publish APIs
- Infra engine uses it for visual regression testing (already has `visual-report` endpoint)

**Tradeoffs:**
- (+) Unlocks scraping behind auth walls (Upwork, LinkedIn) — huge for Harvest
- (+) Content publishing to platforms without APIs
- (+) Visual testing and monitoring
- (−) Chromium is ~200MB — bloats the single-binary story (must be a runtime dependency, not embedded)
- (−) Browser automation is fragile (selectors break, CAPTCHAs, anti-bot)
- (−) Resource-heavy: each browser instance uses 200-500MB RAM

**Mitigation:** Make browser an optional runtime dependency (`rusvel --with-browser`). Default to API-based scraping where possible. Use browser only as fallback for auth-walled or API-less targets.

---

### 4. LIVE CANVAS / A2UI (Agent-Driven UI)

**What OpenClaw has:** Agent pushes HTML/CSS/JS to a canvas workspace. The AI controls what you see — interactive dashboards, forms, visualizations rendered in real-time.

**What it adds to RUSVEL:**
- Agent-generated dashboards: Finance engine pushes a P&L chart mid-conversation
- Interactive forms: "Fill out this contract template" → agent renders editable form
- Workflow visualization: Agent shows pipeline diagram while explaining it
- Data exploration: Growth engine renders cohort chart, user clicks to drill down

**Recommendation: PARTIAL — Borrow the concept, not the implementation**

**Reasoning:**
- RUSVEL already has a SvelteKit frontend with D3, LayerChart, and XYFlow. It doesn't need OpenClaw's Lit/Angular-based A2UI renderer.
- The *concept* of agent-pushed UI is valuable: let the agent decide what to show, not just what to say.
- Implement as a `canvas` tool in the agent runtime that emits structured UI commands (show_chart, show_form, show_table) which the SvelteKit frontend renders using its existing component library.

**How to integrate:**
- Add `AgentUiAction` variants to the event system: `ShowChart { data, type }`, `ShowForm { schema }`, `ShowTable { rows, columns }`, `ShowDiagram { nodes, edges }`
- Frontend SSE stream picks up these events and renders them inline in the chat
- No new rendering engine needed — reuse existing SvelteKit components

**Tradeoffs:**
- (+) Rich agent responses beyond plain text
- (+) Uses existing frontend stack (no new dependencies)
- (−) Limited compared to arbitrary HTML (OpenClaw's approach)
- (−) Structured UI commands need a schema — more upfront design work

---

### 5. DEVICE PAIRING (Mobile Nodes)

**What OpenClaw has:** iOS/Android apps pair with the gateway. Nodes declare capabilities (camera, canvas, screen, location) and respond to commands.

**What it adds to RUSVEL:**
- Photo receipts: Snap expense receipt → Finance engine OCRs and logs it
- Location-aware: "What clients are near me?" → GTM checks contacts by proximity
- Mobile approvals: Rich approval UI on phone (not just chat replies)
- Camera for visual reports: Photograph whiteboard → Content engine extracts notes

**Recommendation: NO — Skip for now**

**Reasoning:**
- Building native mobile apps is a massive undertaking (Swift + Kotlin + pairing protocol)
- The messaging integration (Feature #1) already gives mobile access — reply from Telegram/WhatsApp
- A future PWA version of the SvelteKit frontend handles 80% of mobile needs without native apps
- Camera/location can be handled through the web frontend when needed

---

### 6. MEMORY SYSTEM (Embeddings + RAG + Vector DB)

**What OpenClaw has:** SQLite + sqlite-vec for embeddings, LanceDB support, FTS5 full-text search, chunk-based RAG.

**What it adds to RUSVEL:** RUSVEL already has LanceDB + FastEmbed + FTS5. **No gap here.**

**Recommendation: NO — Already covered**

RUSVEL's `rusvel-vector` (LanceDB) + `rusvel-embed` (FastEmbed) + `rusvel-memory` (FTS5) already match or exceed OpenClaw's memory capabilities. No integration needed.

---

### 7. SECURITY MODEL (DM Pairing + Role-Based Access)

**What OpenClaw has:** Unknown senders blocked by default (pairing code required), device signatures, operator vs node roles with explicit scopes.

**What it adds to RUSVEL:**
- If messaging channels are added (#1), RUSVEL needs sender verification
- Prevent random Telegram users from accessing your CRM/Finance data
- Role-based access if RUSVEL ever supports team mode

**Recommendation: YES — Required if messaging is added**

**How to integrate:**
- Add `SecurityPort` trait to rusvel-core with `verify_sender`, `check_permission` methods
- Implement pairing store in rusvel-db (contacts table with approval status)
- First message from unknown sender triggers pairing flow
- Approved contacts mapped to permission levels (read-only, chat, approve, admin)

**Tradeoffs:**
- (+) Essential for any externally-facing interface
- (+) Protects sensitive financial/CRM data
- (−) Extra friction for legitimate first-time contacts
- (−) Pairing UX varies by channel (some don't support interactive flows)

---

### 8. PLUGIN ARCHITECTURE FOR CHANNELS

**What OpenClaw has:** 40+ channel plugins with a standardized adapter interface (auth, config, messaging, security). Plugins ship as npm packages.

**What it adds to RUSVEL:**
- Standard way to add new messaging channels
- Community can contribute channel plugins
- Each channel isolated (one failing doesn't crash others)

**Recommendation: YES — Adopt the pattern, not the code**

**How to integrate:**
- Define a `ChannelPlugin` trait in rusvel-core:
  ```rust
  trait ChannelPlugin: Send + Sync {
      fn name(&self) -> &str;
      fn connect(&self, config: &Value) -> Result<()>;
      fn send(&self, target: &str, message: Content) -> Result<()>;
      fn recv(&self) -> Result<InboundMessage>;
      fn disconnect(&self) -> Result<()>;
  }
  ```
- Register plugins in the DepartmentRegistry pattern (TOML-based config)
- Bridge to OpenClaw's Node.js plugins via subprocess for initial implementation
- Replace with Rust-native plugins over time

---

## Integration Roadmap

### Phase 1: Channel Foundation (2-3 weeks)
1. Add `ChannelPort` + `SecurityPort` traits to rusvel-core
2. Implement Telegram channel (simplest auth: bot token)
3. Wire inbound messages → department chat router
4. Wire outbound notifications ← event hooks
5. Add sender verification (pairing store)

### Phase 2: Browser + Rich Notifications (2-3 weeks)
1. Add `rusvel-browser` crate (chromiumoxide)
2. Wire Harvest engine to browser for auth-walled scraping
3. Add agent UI actions to event system
4. Frontend renders inline charts/tables/forms from agent events
5. Add Slack channel (OAuth, richer message formatting)

### Phase 3: Full Channel Suite (2-3 weeks)
1. Add WhatsApp channel (most complex: QR pairing via Baileys bridge)
2. Add Discord channel
3. Channel-specific message formatting (Slack blocks, Telegram Markdown, Discord embeds)
4. Approval workflow via channel replies
5. Daily mission push to preferred channel

---

## What NOT to Take from OpenClaw

| Feature | Reason to Skip |
|---------|---------------|
| Voice/Wake Word | UX layer, not a capability gap. Native platform dependency. |
| Device Pairing / Mobile Nodes | Massive scope. Messaging channels give mobile access already. |
| A2UI Renderer (Lit/Angular) | RUSVEL has SvelteKit. Borrow the concept, use existing components. |
| Multi-Agent Routing | RUSVEL's department routing already covers this pattern. |
| Tailscale Integration | RUSVEL is local-first single binary. Reverse proxy is simpler. |
| WebChat Server | RUSVEL already has embedded SvelteKit frontend. |
| Skill Registry (ClawHub) | RUSVEL has its own capability engine. Different distribution model. |
| Session Model | RUSVEL's Session→Run→Thread is more structured than OpenClaw's JSONL. Keep RUSVEL's. |

---

## Architecture Decision: Bridge vs Rewrite

**Option A: Node.js subprocess bridge to OpenClaw plugins**
- (+) Immediate access to 20+ channels
- (+) Battle-tested code (OpenClaw is production-grade)
- (−) Node.js runtime dependency (breaks single-binary)
- (−) IPC overhead, harder to debug
- (−) Tied to OpenClaw's release cycle

**Option B: Rust-native channel implementations**
- (+) Single binary preserved
- (+) Full control, type safety, performance
- (−) Massive rewrite effort (WhatsApp alone is ~10K lines)
- (−) Missing ecosystem (no Rust Baileys equivalent for WhatsApp)

**Option C: Hybrid — Bridge now, rewrite incrementally** ← Recommended
- Start with Node.js bridge for WhatsApp (no Rust alternative)
- Write Telegram + Discord natively in Rust (good crate ecosystem: `teloxide`, `serenity`)
- Slack via REST API (no persistent connection needed, pure Rust HTTP)
- Over time, replace bridges with native implementations as Rust ecosystem matures

**Tradeoffs of Hybrid:**
- (+) Ship fast with battle-tested WhatsApp support
- (+) Native Rust for channels with good crate support
- (+) Incremental migration path
- (−) Two runtime dependencies during transition (Rust + Node.js for WhatsApp bridge)
- (−) Must maintain bridge protocol stability

---

## Summary: Recommended Integrations

| # | Feature | Priority | Effort | Value |
|---|---------|----------|--------|-------|
| 1 | **Multi-channel messaging** (Telegram, Slack, WhatsApp) | High | Large | Transforms RUSVEL from dashboard to conversational assistant |
| 2 | **Sender security model** (pairing + permissions) | High | Medium | Required for #1 to be safe |
| 3 | **Browser automation** (CDP) | Medium | Medium | Unlocks auth-walled scraping + API-less publishing |
| 4 | **Agent UI actions** (concept from Canvas) | Medium | Small | Rich agent responses using existing frontend |
| 5 | **Channel plugin trait** (pattern from OpenClaw) | Medium | Small | Clean architecture for channel extensibility |

**Total estimated new crates:** 3 (`rusvel-channels`, `rusvel-browser`, `rusvel-security`)
**New port traits:** 3 (`ChannelPort`, `BrowserPort`, `SecurityPort`)
**Existing infrastructure reused:** Event hooks, department chat, job queue, approval workflows

The messaging integration alone would be transformative — turning every department into something you can talk to from your phone while walking, not just from a browser at your desk.
