# RUSVEL Onboarding, Guidance & Documentation Strategy

> Goal: Make the first 60 seconds magical, then get out of the way.

## Stack

| Layer | Tool | License | Size | Why |
|-------|------|---------|------|-----|
| Doc site | Starlight (Astro) | MIT | Static | Free, fast, MDX, embeds Svelte, minimal maintenance |
| Product tour | Driver.js | MIT | ~5KB | Zero deps, works with SvelteKit, clean highlight API |
| Command palette | cmdk-sv | MIT | ~8KB | Svelte port of Linear/Vercel cmdk, unstyled for Tailwind |
| CLI wizard | cliclack | MIT | Rust | Beautiful first-run prompts, Railway/Vercel aesthetic |
| API docs | utoipa + Swagger UI | MIT | Rust | Auto-generate OpenAPI from Axum routes |
| Contextual help | Custom (FTS5 + agent) | — | — | Leverages existing rusvel-memory + rusvel-agent |

---

## P1: CLI First-Run Wizard

**Dep:** `cliclack` crate
**Where:** `crates/rusvel-app/src/main.rs` — after `seed_defaults()` (line ~280), before `Cli::parse()` (line ~295)

**Why here:** This is the only point where all adapters are initialized but no action has been taken yet. Profile is loaded right after seed_defaults — if no `~/.rusvel/profile.toml` exists, we intercept and run the wizard instead of silently setting `profile = None`.

**What it does:**
```
$ rusvel
  Welcome to RUSVEL — Your AI-Powered Virtual Agency

  ✓ Ollama detected at localhost:11434
  ? Your name: Mehdi
  ? Create your first session? (Y/n) Y
  ? Session name: my-startup

  ✓ Session "my-startup" created
  ✓ API server running at http://localhost:3000

  Next steps:
    → Open http://localhost:3000 in your browser
    → Or run: rusvel forge mission today
```

**Integration details:**
- New function `first_run_wizard()` in `main.rs` (or a new `crates/rusvel-onboard/` crate if >200 lines)
- Guard: `if !profile_path.exists() && atty::is(atty::Stream::Stdin)` — only runs in interactive terminal, not CI
- Creates minimal `profile.toml` with name + role
- Creates first session via `SessionPort::create()`
- Writes `active_session` file
- Skipped entirely with `--yes` or `--non-interactive` CLI flag (add to Clap struct)
- Skipped if any subcommand is passed (user already knows what they want)

**Reason:** First impression defines retention. A blank terminal prompt with `--help` output loses users. cliclack gives Railway/Vercel-quality UX in ~50 lines of Rust.

---

## P2: Frontend Empty States

**Dep:** None (use existing `EmptyState.svelte` component at `frontend/src/lib/components/ui/EmptyState.svelte`)

**Where:** Multiple files, each with specific insertion points:

### Dashboard (`frontend/src/routes/+page.svelte`)
- **Line 49-53** (no session selected): Replace plain text with `EmptyState` component showing session creation form inline
- **Line 101-104** (no goals): Add "Create your first goal" button that navigates to `/forge`
- **Line 130-131** (no events): Add explanation of what events are + "Run a mission to see events here"

### Department Pages (all 12 `frontend/src/routes/[dept]/+page.svelte`)
- **No-session guard** (line ~19): Replace "Select a session" text with `EmptyState` showing session picker + explanation of what this department does

### DepartmentChat (`frontend/src/lib/components/chat/DepartmentChat.svelte`)
- **Line 180-188** (empty chat): Currently shows generic "ready to work" — enhance with 3-4 department-specific suggested prompts (like the God Agent chat already does at `chat/+page.svelte` line 201-208)

### DepartmentPanel tabs (`frontend/src/lib/components/chat/DepartmentPanel.svelte`)
- **Lines 340, 412, 461, 493, 529, 569, 594**: Each empty tab already has text like "No agents. Create one above." — add a primary CTA button that opens the create form or scrolls to it

**Reason:** Empty screens are the #1 killer of new user activation. Every blank state is an opportunity to teach what belongs there and how to fill it. The `EmptyState.svelte` component already exists but isn't used — it has icon, title, description, and actions slot built in.

---

## P3: Onboarding Checklist

**Dep:** None (pure Svelte + localStorage)
**Where:** New component `frontend/src/lib/components/onboarding/OnboardingChecklist.svelte`, mounted in `frontend/src/routes/+layout.svelte`

**Mount point in layout:** After the sidebar, before `{@render children()}` — as a floating card anchored to bottom-left of the main content area (z-20, position fixed).

**Why in layout, not dashboard:** The checklist should persist across page navigations. User might create a session on dashboard, then navigate to forge for the next step.

**Checklist steps:**
1. ✅ Create your first session — auto-checked when `$sessions.length > 0`
2. ⬜ Add a goal — checked when API returns goals for active session
3. ⬜ Generate daily plan — checked when user runs mission/today
4. ⬜ Chat with a department — checked when any dept conversation exists
5. ⬜ Create an agent — checked when user-created agent exists (not seeded)

**State tracking:**
- Store completion in `localStorage('rusvel-onboarding')` as JSON `{ step1: true, ... dismissed: false }`
- New store in `frontend/src/lib/stores.ts`: `onboardingState`
- Dismiss button sets `dismissed: true`, hides permanently
- Progress bar at top showing "2/5 complete"

**Reason:** Checklists have the highest completion rate of any onboarding pattern. They create a sense of progress and give users a clear path through the app without blocking them.

---

## P4: Command Palette (Cmd+K)

**Dep:** `cmdk-sv` npm package
**Where:** New component `frontend/src/lib/components/CommandPalette.svelte`, mounted in `frontend/src/routes/+layout.svelte`

**Mount point in layout:** At the end of the layout, after `{@render children()}` — renders as a modal overlay (z-50, fixed positioning). Same pattern as all command palettes.

**Why cmdk-sv:** It's the Svelte port of the cmdk library used by Linear, Vercel, and Raycast. Unstyled (matches our Tailwind design system), composable, accessible, keyboard-native.

**Command groups:**

```
Navigation
  → Dashboard          (goto /)
  → Chat (God Agent)   (goto /chat)
  → Forge Department   (goto /forge)
  → ... all 12 departments
  → Settings           (goto /settings)

Actions
  → Create Session     (opens session form)
  → Add Goal           (opens goal form in forge)
  → Generate Daily Plan (triggers mission/today API)
  → New Chat           (starts new conversation)

Search
  → Search agents...   (filters from /api/agents)
  → Search skills...   (filters from /api/skills)
```

**Integration:**
- Global `keydown` listener in layout: `Cmd+K` / `Ctrl+K` toggles visibility
- Navigation commands use SvelteKit `goto()`
- Action commands dispatch to existing API functions in `lib/api.ts`
- Search commands fetch and filter client-side

**Reason:** Power users expect Cmd+K. With 14+ navigation targets and growing CRUD entities, a command palette prevents the sidebar from becoming overwhelming. It also serves as a discoverability tool — users find features they didn't know existed.

---

## P5: Starlight Doc Site

**Dep:** `@astrojs/starlight` (separate project)
**Where:** New directory `docs-site/` at repo root (sibling to `frontend/` and `crates/`)

**Why separate directory:** Doc site has its own build pipeline (Astro), its own deployment (static hosting), and its own dependencies. Mixing it into the SvelteKit frontend would complicate both build systems.

**Why Starlight over alternatives:**
- Mintlify: $150/month, vendor lock-in — wrong for solo builder
- Docusaurus: React-based, heavy, maintenance burden
- VitePress: Vue-based — adds unnecessary framework
- Starlight: Astro (can embed Svelte components), zero JS by default, free, self-hostable

**Site structure:**
```
docs-site/
├── astro.config.mjs
├── package.json
└── src/
    └── content/
        └── docs/
            ├── index.md                    # Landing / hero
            ├── getting-started/
            │   ├── installation.md         # Build from source, install binary
            │   ├── first-run.md            # CLI wizard, create session
            │   └── first-mission.md        # Generate daily plan, see results
            ├── concepts/
            │   ├── sessions.md             # What sessions are, kinds, lifecycle
            │   ├── departments.md          # The 12 departments, what each does
            │   ├── agents.md               # Agent profiles, personas, how they work
            │   ├── skills.md               # Skill definitions, how they're used
            │   ├── rules.md                # Rules injection into system prompts
            │   └── workflows.md            # Sequential, parallel, loop, graph patterns
            ├── departments/
            │   ├── forge.md                # Mission planning, goals, reviews
            │   ├── code.md                 # Code intelligence, parsing, search
            │   ├── content.md              # Content creation, calendar, publishing
            │   ├── harvest.md              # Opportunity discovery, pipeline
            │   ├── gtm.md                  # CRM, outreach, invoicing
            │   ├── finance.md              # Ledger, runway, tax
            │   ├── product.md              # Roadmap, pricing, feedback
            │   ├── growth.md               # KPIs, funnels, cohorts
            │   ├── distro.md               # SEO, marketplace, affiliate
            │   ├── legal.md                # Contracts, compliance, IP
            │   ├── support.md              # Tickets, knowledge base, NPS
            │   └── infra.md               # Deploy, monitoring, incidents
            ├── reference/
            │   ├── cli.md                  # All CLI commands + flags
            │   ├── api.md                  # REST API endpoints (from OpenAPI)
            │   └── configuration.md        # config.toml + profile.toml reference
            └── architecture/
                ├── overview.md             # Hexagonal architecture, ports & adapters
                ├── decisions.md            # ADRs (link to existing docs/design/decisions.md content)
                └── self-building.md        # How RUSVEL develops itself via chat
```

**Deployment:** Static build → GitHub Pages or Cloudflare Pages (free tier, zero config)

**Cross-linking:** Frontend help tooltips and empty states link to doc site URLs. CLI `--help` output includes doc site URL.

**Reason:** A doc site is table stakes for any tool that wants adoption. It serves SEO (people find RUSVEL via search), credibility (looks professional), and self-service (users solve their own problems). Starlight builds in <5 seconds and deploys for free.

---

## P6: Driver.js Product Tour

**Dep:** `driver.js` npm package
**Where:** New component `frontend/src/lib/components/onboarding/ProductTour.svelte`, mounted in `frontend/src/routes/+layout.svelte`

**Mount point in layout:** Same as command palette — end of layout, modal overlay. Initialize in `onMount()` to avoid SSR issues.

**Trigger:** First visit detected via `localStorage('rusvel-tour-completed')`. Shows a "Take a tour?" prompt (non-blocking) — user can skip.

**Tour steps:**

| Step | Target Element | Content |
|------|---------------|---------|
| 1 | Sidebar logo `.sidebar-logo` | "Welcome to RUSVEL — your AI-powered virtual agency" |
| 2 | Session switcher `.session-switcher` | "Start by creating a session — it's your workspace for goals and plans" |
| 3 | Nav item: Forge `.nav-forge` | "Forge is your mission control — set goals and generate daily plans" |
| 4 | Nav item: Chat `.nav-chat` | "Chat with the God Agent — it has authority over all departments" |
| 5 | Nav item: Settings `.nav-settings` | "Check system health and engine status here" |

**Integration:**
- Add `data-tour="sidebar-logo"` attributes to target elements in `+layout.svelte`
- Driver.js uses CSS selectors, so we add semantic class names or data attributes
- Tour completion stored in localStorage
- "Restart tour" option in Settings page

**Why Driver.js over alternatives:**
- Shepherd.js: AGPL license — requires commercial license for closed-source use
- Intro.js: AGPL same issue
- Driver.js: MIT, 5KB, zero deps, works with any framework

**Reason:** Product tours have the second-highest activation rate after checklists. They're especially effective for spatial interfaces (sidebars, panels) where users need to build a mental map of where things are.

---

## P7: AI-Powered /help in Chat

**Dep:** None (uses existing `rusvel-memory` FTS5 + `rusvel-agent` runtime)
**Where:**
- Backend: New route `POST /api/help` in `crates/rusvel-api/src/routes.rs`
- Frontend: Intercept `/help` prefix in chat input across `DepartmentChat.svelte` and `chat/+page.svelte`

**How it works:**
1. User types `/help how do workflows work?` in any chat
2. Frontend detects `/help` prefix, calls `POST /api/help` instead of regular chat
3. Backend indexes RUSVEL's own `docs/` markdown files into FTS5 memory store (on first `/help` call, cached)
4. Agent receives the question + relevant doc snippets as context
5. Response streamed back through existing SSE infrastructure

**Why custom over a docs chatbot:**
- RUSVEL already has FTS5 search (`rusvel-memory`) and agent runtime (`rusvel-agent`)
- No external dependency needed — it's self-hosted, self-aware help
- Context-aware: the agent knows which department the user is in, what session is active
- Aligns with the "self-building" vision — RUSVEL explains itself

**Integration detail:**
- Add `help_handler()` in `routes.rs` alongside existing `chat_handler()`
- Reuse `streamDeptChat()` in frontend with a flag for help mode
- Index docs lazily: first `/help` call triggers `docs/**/*.md` ingestion into memory store
- Add `/help` as a suggested prompt in empty states

**Reason:** AI-powered help is the natural evolution of documentation for an AI-native tool. Users shouldn't have to leave the app to understand the app. This also dogfoods RUSVEL's own agent + memory infrastructure.

---

## P8: Contextual ? Tooltips

**Dep:** None (use existing `Tooltip.svelte` component at `frontend/src/lib/components/ui/Tooltip.svelte`)
**Where:** `frontend/src/lib/components/chat/DepartmentPanel.svelte` header section (line ~251-269)

**What it adds:**
- A `?` icon button in each department panel header, next to the collapse button
- On click/hover: popover with:
  - One-line description of what this department does
  - 2-3 example prompts the user can click to send
  - "Learn more →" link to doc site department guide

**Per-department content (passed as props):**

| Dept | Description | Example Prompts |
|------|-------------|-----------------|
| Forge | Mission planning — set goals, generate daily plans, run reviews | "Plan my day", "Add a goal: launch MVP by April" |
| Code | Code intelligence — parse, search, analyze your codebase | "Find all TODO comments", "Show dependency graph" |
| Content | Content creation — write, schedule, publish across platforms | "Draft a blog post about X", "Show content calendar" |
| Harvest | Opportunity discovery — scan sources, score leads, generate proposals | "Scan for new opportunities", "Score this lead" |
| GTM | Go-to-market — CRM, outreach sequences, invoicing | "Create outreach sequence for X", "Generate invoice" |
| Finance | Financial management — ledger, runway tracking, tax prep | "Show current runway", "Log expense" |
| Product | Product management — roadmap, pricing, user feedback | "Show roadmap", "Analyze feedback trends" |
| Growth | Growth analytics — KPIs, funnels, cohort analysis | "Show KPI dashboard", "Analyze last month's funnel" |
| Distro | Distribution — SEO, marketplace listings, affiliate management | "Audit SEO for landing page", "List marketplace channels" |
| Legal | Legal — contracts, compliance, IP management | "Draft NDA template", "Check compliance status" |
| Support | Support — tickets, knowledge base, NPS tracking | "Show open tickets", "What's our NPS score?" |
| Infra | Infrastructure — deployments, monitoring, incident response | "Deploy status", "Show recent incidents" |

**Integration:**
- Add `helpContent` prop to `DepartmentPanel.svelte` (object: `{ description, prompts, docsUrl }`)
- Each department page passes its help content when instantiating `DepartmentPanel`
- Clicking an example prompt dispatches the existing `dept-quick-action` CustomEvent
- "Learn more" links to `https://docs.rusvel.dev/departments/{dept}`

**Reason:** Contextual help at point-of-use has the highest engagement rate of any help format. Users see relevant guidance exactly when they need it, without leaving their workflow. The example prompts also serve as feature discovery — users learn what a department can do by seeing what to ask it.

---

## Design Principles

1. **Value in 60 seconds** — no config files needed for first run
2. **Progressive disclosure** — basics first, advanced features revealed on exploration
3. **Self-documenting** — the app explains itself through empty states and AI help
4. **Non-blocking** — onboarding never prevents the user from doing real work
5. **Dismissible** — every guidance element can be permanently hidden

---

## Dependency Summary

### Rust (Cargo.toml additions)
| Crate | Used by | Purpose |
|-------|---------|---------|
| `cliclack` | rusvel-app | First-run wizard prompts |
| `atty` | rusvel-app | Detect interactive terminal |
| `utoipa` + `utoipa-swagger-ui` | rusvel-api | OpenAPI spec generation |

### Frontend (package.json additions)
| Package | Purpose |
|---------|---------|
| `cmdk-sv` | Command palette |
| `driver.js` | Product tour |

### Separate project (docs-site/)
| Package | Purpose |
|---------|---------|
| `@astrojs/starlight` | Documentation site generator |

---

## Implementation Order Rationale

1. **P1 CLI wizard** → First thing any user touches. Bad first impression = no second chance.
2. **P2 Empty states** → Zero effort, maximum clarity. Uses existing component.
3. **P3 Checklist** → Guides users through the "aha moment" (first plan generation).
4. **P4 Command palette** → Power user retention. Makes the app feel professional.
5. **P5 Doc site** → SEO, credibility, self-service. Required before any public launch.
6. **P6 Product tour** → Nice-to-have polish. Only valuable after empty states and checklist exist.
7. **P7 AI help** → Leverages existing infra, impressive demo, but users need basics first.
8. **P8 Tooltips** → Final polish layer. Only valuable after departments have real functionality.
