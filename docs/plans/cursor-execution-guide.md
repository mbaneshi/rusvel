# Cursor Execution Guide — Supervisor Checklist

> **Role split:** Cursor executes. Mehdi + Claude supervise.
> **Rule:** Never merge a task until every acceptance check passes.
> **Date:** 2026-03-24

---

## Corrected Task Sequence

Incorporates both Claude audit + Cursor counter-review.
Strikethrough = dropped from original plan with reason.

| # | Task | Status |
|---|------|--------|
| ~~P0-1~~ | ~~Unify job store~~ | **ALREADY DONE** — `main.rs:816` already wires `db.clone() as Arc<dyn JobPort>` |
| P0-2 | Fix HarvestScan job (scan not pipeline) | 🔲 |
| ~~P0-3~~ | ~~Fix ObjectStore session_id filter~~ | **ALREADY WORKS** — `store.rs:312` uses `json_extract(data, '$.session_id')` |
| P0-4 | Harden worker: use `job.session_id` not payload parse | 🔲 |
| A1 | CodeAnalysisSummary (project existing CodeAnalysis) | 🔲 |
| A2 | Prompt builder in content-engine writer | 🔲 |
| A3 | POST `/api/dept/content/from-code` endpoint | 🔲 |
| A4 | PATCH `/api/dept/content/{id}/approve` (content-level approval) | 🔲 |
| A5 | LinkedIn PlatformAdapter (real HTTP) | 🔲 |
| A6 | Twitter/X PlatformAdapter (real HTTP) | 🔲 |
| A7 | Integration test: code → content → approve → publish | 🔲 |
| B1 | Wire UserProfile.skills into HarvestConfig | 🔲 |
| B2 | Upwork RSS source adapter | 🔲 |
| B3 | Freelancer RSS source adapter | 🔲 |
| B4 | Persist proposals to object store | 🔲 |
| B5 | ProposalDraft JobKind + worker arm | 🔲 |
| B6 | POST `/api/dept/harvest/scan` endpoint (the missing one) | 🔲 |
| B8 | Integration test: harvest → score → propose | 🔲 |
| C1 | DeployPort in rusvel-core | 🔲 |
| C2 | Fly.io adapter (new `rusvel-deploy` crate, NOT `rusvel-adapters`) | 🔲 |
| C3 | MarketplaceBidPort — **SPIKE FIRST**, then design | 🔲 |
| C4 | BidSubmit JobKind + approval gate | 🔲 |
| C5 | Orchestrated flow via `flow-engine`, NOT single god-endpoint | 🔲 |

---

## Phase 0 — Bug Fixes

### P0-2: Fix HarvestScan job handler

**Tell Cursor:**
> In `crates/rusvel-app/src/main.rs`, find the `JobKind::HarvestScan` match arm.
> It currently calls `harvest_engine_worker.pipeline(&sid)` which just returns stats.
> Change it to call `harvest_engine_worker.scan(&sid, &MockSource::new())`.
> Import MockSource from `harvest_engine::source::MockSource`.
> This is a temporary fix — Phase 2 replaces MockSource with real sources.

**Acceptance criteria:**
- [ ] `JobKind::HarvestScan` arm calls `.scan()` not `.pipeline()`
- [ ] Enqueuing a HarvestScan job actually creates Opportunity objects in the object store
- [ ] `cargo test -p rusvel-app` passes
- [ ] `cargo build` succeeds

**Verify:**
```bash
# Check the change
grep -A 10 "HarvestScan =>" crates/rusvel-app/src/main.rs

# Build
cargo build 2>&1 | tail -5

# Run tests
cargo test 2>&1 | tail -10
```

**Red flags — reject if:**
- Cursor imports `harvest_engine` directly from an engine crate (violates hexagonal)
- Cursor changes the `HarvestEngine::scan()` signature
- Cursor removes the `pipeline()` method

---

### P0-4: Harden worker session_id extraction

**Tell Cursor:**
> In `crates/rusvel-app/src/main.rs`, the job worker extracts session_id from
> `job.payload` via JSON parsing with `unwrap_or_default()`.
> Change all worker arms to use `job.session_id` directly (the first-class field
> on the Job struct) instead of parsing it from the JSON payload.
> Keep `session_id` in payloads for backward compat, but the worker should
> trust the struct field, not the JSON blob.

**Acceptance criteria:**
- [ ] Worker match arms use `job.session_id` not `serde_json::from_str(payload)["session_id"]`
- [ ] No `unwrap_or_default()` for session_id extraction
- [ ] All existing tests pass: `cargo test`
- [ ] Content, Code, and Harvest job arms all updated consistently

**Verify:**
```bash
# Should show job.session_id usage, not payload parsing for session
grep -n "session_id" crates/rusvel-app/src/main.rs | head -20

# All tests
cargo test 2>&1 | tail -10
```

---

## Phase 1 — Use Case A: Codebase → Social Content

### A1: CodeAnalysisSummary

**Tell Cursor:**
> In `crates/rusvel-core/src/domain.rs`, add a `CodeAnalysisSummary` struct.
> But first READ `crates/code-engine/src/lib.rs` — the `CodeAnalysis` struct
> already has `ProjectMetrics` with `total_files`, `total_symbols`,
> `avg_function_length`, `largest_function`, and `Vec<Symbol>`.
> `CodeAnalysisSummary` should be a **flattened projection** of that existing data,
> not a duplicate. Add a `From<CodeAnalysis> for CodeAnalysisSummary` impl.
> Fields: snapshot_id, repo_path, total_files, total_symbols, top_symbols (Vec<String>),
> largest_function (Option<String>).
> Derive: Serialize, Deserialize, Debug, Clone.

**Acceptance criteria:**
- [ ] Struct in `rusvel-core/src/domain.rs`
- [ ] `From<CodeAnalysis>` impl exists (in code-engine, not core — core doesn't know about CodeAnalysis)
- [ ] Derives: Serialize, Deserialize, Debug, Clone
- [ ] `cargo test -p rusvel-core` passes
- [ ] `cargo test -p code-engine` passes

**Red flag:** If Cursor puts the `From` impl in `rusvel-core` instead of `code-engine`, reject — core can't depend on an engine.

---

### A2: Prompt builder in content-engine writer

**Tell Cursor:**
> In `crates/content-engine/src/writer.rs`, add:
> `pub fn build_code_prompt(summary: &CodeAnalysisSummary, kind: &ContentKind) -> String`
> This returns a prompt string (not an LLM call). The caller passes it to `draft()` as the topic.
> For LinkedInPost: hook line + 3 key stats + what makes the codebase interesting + CTA.
> For Thread: 6-8 tweet-sized chunks, each a single insight.
> For Blog: full technical writeup structure with sections.
> Keep existing `draft()` method UNCHANGED.

**Acceptance criteria:**
- [ ] Function is pure (no async, no ports, just string building)
- [ ] Handles at least: `ContentKind::LinkedInPost`, `ContentKind::Thread`, `ContentKind::Blog`
- [ ] `draft()` method signature unchanged
- [ ] Has unit tests for each content kind
- [ ] `cargo test -p content-engine` passes

**Verify:**
```bash
cargo test -p content-engine 2>&1 | tail -15
```

---

### A3: POST `/api/dept/content/from-code` endpoint

**Tell Cursor:**
> In `crates/rusvel-api/src/engine_routes.rs`, add handler for
> `POST /api/dept/content/from-code`.
> Request: `{ session_id, path, kinds: ["LinkedInPost", "Thread"] }`
> Logic: call code_engine.analyze(path) → build CodeAnalysisSummary →
> for each kind: call content_engine.draft() with build_code_prompt() result as topic.
> Register in `crates/rusvel-api/src/lib.rs` router.
> IMPORTANT: route prefix is `/api/dept/` not `/api/engine/`.

**Acceptance criteria:**
- [ ] Route is `/api/dept/content/from-code` (not `/api/engine/`)
- [ ] Handler takes `State<Arc<AppState>>` — check how other engine_routes handlers work
- [ ] Returns `Vec<ContentItem>` as JSON
- [ ] Registered in `lib.rs` router
- [ ] `cargo test -p rusvel-api` passes
- [ ] `cargo build` succeeds

**Red flag:** If the handler imports code-engine directly instead of going through AppState, reject.

---

### A4: Content-level approval endpoint

**Tell Cursor:**
> This is DIFFERENT from `/api/approvals/{id}/approve` (which approves Jobs).
> ContentEngine::publish() gates on `ContentItem.approval == ApprovalStatus::Approved`.
> Add `PATCH /api/dept/content/{id}/approve` that:
> 1. Loads ContentItem from object store by id
> 2. Sets `item.approval = ApprovalStatus::Approved`
> 3. Saves back to object store
> 4. Returns 200 with updated item
> Put it in `crates/rusvel-api/src/engine_routes.rs` (or new `content_routes.rs`).

**Acceptance criteria:**
- [ ] Route is PATCH, not POST
- [ ] Loads from and saves to ObjectStore (not job store)
- [ ] Sets `ApprovalStatus::Approved` on the ContentItem struct
- [ ] 404 if content item not found
- [ ] `cargo test -p rusvel-api` passes

**Red flag:** If Cursor confuses this with job approval and modifies `approvals.rs`, reject.

---

### A5 & A6: LinkedIn + Twitter adapters

**Tell Cursor:**
> In `crates/content-engine/src/adapters/`, create `linkedin.rs` and `twitter.rs`.
> Both implement the `PlatformAdapter` trait from `crates/content-engine/src/platform.rs`.
> Check the trait signature: `publish()`, `metrics()`, `max_length()`, `format_content()`.
> LinkedIn: POST to `https://api.linkedin.com/v2/ugcPosts`, Bearer token from ConfigPort key "linkedin_token".
> Twitter: POST to `https://api.twitter.com/2/tweets`, Bearer token from ConfigPort key "twitter_token".
> Export from `adapters/mod.rs`. Wire in `main.rs` ContentEngine construction.
> Add `reqwest` to content-engine's Cargo.toml if not already there.

**Acceptance criteria:**
- [ ] Both implement `PlatformAdapter` trait exactly
- [ ] Auth tokens loaded from ConfigPort, not hardcoded
- [ ] Error handling for rate limits (429 status)
- [ ] Exported from `adapters/mod.rs`
- [ ] Registered in `main.rs` ContentEngine construction
- [ ] `cargo build` succeeds
- [ ] Unit tests with mock HTTP (don't need real API keys to pass tests)

**Red flag:** If Cursor puts API keys as string literals or skips error handling, reject.

---

### A7: Integration test

**Tell Cursor:**
> Create `crates/rusvel-api/tests/code_to_content.rs`.
> FIRST: read existing test files in `crates/rusvel-api/tests/` or `src/` test modules
> to understand the test infrastructure (how AppState is built, how routes are called).
> Test 1: POST /dept/content/from-code → ContentItems created with correct kinds
> Test 2: PATCH /dept/content/{id}/approve → approval status flipped
> Test 3: ContentPublish job → worker calls MockPlatformAdapter
> Use mock agents and mock platform adapters — no real LLM or API calls.

**Acceptance criteria:**
- [ ] Uses same test infrastructure as existing API tests
- [ ] All 3 test cases pass
- [ ] No real HTTP calls (mocked)
- [ ] `cargo test -p rusvel-api` passes (all tests, including existing ones)

---

## Phase 2 — Use Case B: Harvest → Score → Propose

### B1: Wire UserProfile.skills into HarvestConfig

**Tell Cursor:**
> In `crates/rusvel-app/src/main.rs`, where HarvestEngine is constructed (around line 852),
> load skills from ConfigPort instead of using the default `vec!["rust"]`.
> Check `rusvel-config` for how to read config values.
> If no config key exists, fall back to the current default.

**Acceptance criteria:**
- [ ] HarvestConfig.skills populated from config, not hardcoded
- [ ] Graceful fallback if config key missing
- [ ] `cargo build` succeeds

---

### B2 & B3: Upwork + Freelancer RSS sources

**Tell Cursor:**
> In `crates/harvest-engine/src/source.rs` (or new files in a `sources/` directory),
> create `UpworkRssSource` and `FreelancerRssSource`.
> Both should wrap the existing `RssSource` internally.
> Upwork RSS: `https://www.upwork.com/ab/feed/jobs/rss?q={query}&sort=recency`
> Freelancer: `https://www.freelancer.com/rss/projects?query={query}`
> Constructor takes `query: String, skills: Vec<String>`.
> `source_kind()` returns `OpportunitySource::Upwork` / `OpportunitySource::Freelancer`.
> IMPORTANT: RssSource currently doesn't extract budget or skills from feed items.
> Add basic regex extraction from the description HTML for budget ranges and skill tags.

**Acceptance criteria:**
- [ ] Both implement `HarvestSource` trait
- [ ] Wrap `RssSource` internally (don't duplicate XML parsing)
- [ ] URL construction is correct with proper query encoding
- [ ] Budget/skills extraction from description (best effort, regex is fine)
- [ ] Unit tests with sample RSS XML (no live HTTP)
- [ ] `cargo test -p harvest-engine` passes (all 15+ existing tests still green)

---

### B4: Persist proposals to object store

**Tell Cursor:**
> In `crates/harvest-engine/src/lib.rs`, the `generate_proposal()` method calls
> ProposalGenerator::generate() but doesn't persist the result.
> After generation, store to ObjectStore with:
> - kind: "proposal"
> - key: `{opportunity_id}_{timestamp}`
> - Emit event `harvest.proposal.persisted`
> Also add `get_proposals(session_id) -> Vec<Proposal>` method to HarvestEngine
> that lists from object store with kind="proposal" filtered by session_id.

**Acceptance criteria:**
- [ ] Proposal stored to ObjectStore after generation
- [ ] `get_proposals()` returns correct results filtered by session
- [ ] Event emitted
- [ ] Existing tests still pass
- [ ] New test: generate_proposal → get_proposals returns it

---

### B5: ProposalDraft JobKind

**Tell Cursor:**
> In `crates/rusvel-core/src/domain.rs`, add `ProposalDraft` to the `JobKind` enum.
> In `crates/rusvel-app/src/main.rs`, add a match arm in the job worker:
> - Extract `opportunity_id` and `profile` from `job.payload`
> - Use `job.session_id` for the session (per P0-4 pattern)
> - Call `harvest_engine.generate_proposal(session_id, opportunity_id, profile)`
> - After generation, set job status to `AwaitingApproval`

**Acceptance criteria:**
- [ ] `ProposalDraft` variant added to `JobKind` enum
- [ ] Worker arm uses `job.session_id` (not payload extraction)
- [ ] Sets `AwaitingApproval` after proposal generation
- [ ] `cargo build` succeeds
- [ ] `cargo test` passes

---

### B6: POST `/api/dept/harvest/scan` endpoint

**Tell Cursor:**
> This is the CRITICAL missing endpoint.
> In `crates/rusvel-api/src/engine_routes.rs`, add:
> `POST /api/dept/harvest/scan`
> Request: `{ session_id, sources: ["upwork", "freelancer"], query: "rust developer" }`
> Logic: for each source string, instantiate the corresponding HarvestSource,
> call harvest_engine.scan(session_id, &source), collect results.
> Returns: `Vec<Opportunity>` (scored and persisted).
> Register in router.

**Acceptance criteria:**
- [ ] Route path is `/api/dept/harvest/scan`
- [ ] Accepts source list + query
- [ ] Returns scored opportunities
- [ ] `cargo test -p rusvel-api` passes
- [ ] New opportunities appear in object store after call

---

### B8: Integration test

**Tell Cursor:**
> Create `crates/rusvel-api/tests/harvest_to_proposal.rs`.
> Test 1: POST /dept/harvest/scan with MockSource → opportunities persisted, scored
> Test 2: POST /dept/harvest/proposal → proposal returned and persisted
> Test 3: Verify session_id isolation (scan session A, query session B → empty)

**Acceptance criteria:**
- [ ] All 3 tests pass
- [ ] Session isolation verified
- [ ] Uses MockSource (no real HTTP)
- [ ] `cargo test -p rusvel-api` — all tests pass

---

## Phase 3 — Demo + Deploy + Bid

### C1: DeployPort

**Tell Cursor:**
> In `crates/rusvel-core/src/ports.rs`, add:
> ```rust
> #[async_trait]
> pub trait DeployPort: Send + Sync {
>     async fn deploy(&self, artifact_path: &Path, service_name: &str) -> Result<DeployedUrl>;
>     async fn status(&self, deployment_id: &str) -> Result<DeployStatus>;
> }
> ```
> Add `DeployedUrl { url: String, deployment_id: String }` and
> `DeployStatus { id: String, state: String, url: Option<String> }` to `domain.rs`.
> Derive Serialize, Deserialize, Debug, Clone on both.

**Acceptance criteria:**
- [ ] Trait in `ports.rs` with `#[async_trait]`
- [ ] Domain types in `domain.rs`
- [ ] `cargo test -p rusvel-core` passes

---

### C2: Fly.io adapter — NEW CRATE `rusvel-deploy`

**Tell Cursor:**
> Create a new crate: `crates/rusvel-deploy/`.
> NOT `rusvel-adapters` — that doesn't exist.
> Implement `DeployPort` using `flyctl` CLI via `tokio::process::Command`.
> Read `FLY_API_TOKEN` from ConfigPort.
> Wire in `main.rs`.
> Add to workspace `Cargo.toml`.

**Acceptance criteria:**
- [ ] New crate at `crates/rusvel-deploy/`
- [ ] Added to workspace `Cargo.toml` members
- [ ] Implements `DeployPort` trait
- [ ] Uses `tokio::process::Command` for flyctl (not raw HTTP to Machines API)
- [ ] Auth from ConfigPort, not hardcoded
- [ ] Wired in `main.rs`
- [ ] `cargo build` succeeds
- [ ] Crate < 2000 lines

---

### C3: MarketplaceBidPort — RESEARCH SPIKE

**Tell Cursor:**
> DO NOT implement yet. Create a SPIKE document first.
> Create `docs/spikes/marketplace-bid-api-research.md` with:
> 1. Can Upwork API actually submit proposals? (OAuth scopes, account type requirements)
> 2. Can Freelancer API submit bids? (endpoint: `/projects/{id}/bids`)
> 3. What auth is needed for each?
> 4. Rate limits and restrictions
> 5. Recommendation: stub trait now or wait?
> After the spike, we decide on C3/C4/C5 design.

**Acceptance criteria:**
- [ ] Research doc exists with real API documentation links
- [ ] Honest assessment of feasibility per platform
- [ ] No code changes in this task

---

## Global Verification Commands

Run these after EVERY task:

```bash
# Must always pass
cargo build 2>&1 | tail -5
cargo test 2>&1 | tail -15

# Architecture check: engines must never import adapters
grep -r "rusvel-db\|rusvel-llm\|rusvel-agent\|rusvel-auth" crates/*-engine/Cargo.toml
# ^ Should return NOTHING. If it does, reject.

# Line count check: no crate > 2000 lines
find crates/ -name "*.rs" | while read f; do
  dir=$(echo "$f" | cut -d/ -f1-3)
  wc -l "$f"
done | sort -rn | head -20

# Check hexagonal boundary
grep -rn "use rusvel_db\|use rusvel_llm\|use rusvel_agent" crates/*-engine/src/
# ^ Should return NOTHING.
```

---

## When to Escalate to Claude

Bring it back to me (Claude in Cowork) when:

1. **Cursor changes a port trait signature** — I need to verify all implementors still compile
2. **Cursor adds a dependency between engine crates** — architecture violation
3. **Cursor proposes a schema migration** — I need to verify backward compat
4. **Tests fail and Cursor's fix looks suspicious** — paste the error, I'll diagnose
5. **Any task touches `main.rs` composition root** — I'll verify the wiring
6. **Before merging Phase 1 or Phase 2** — I'll do a full review pass

---

## Task Priority Order (execute in this sequence)

```
P0-2 → P0-4 → A1 → A2 → A3 → A4 → A5 → A6 → A7
                                                  ↓
                              B1 → B2 → B3 → B4 → B5 → B6 → B8
                                                              ↓
                                              C1 → C2 → C3 (spike) → decide C4/C5
```

Phase 1 and Phase 2 can run in parallel after P0 is done,
as long as they don't touch the same files simultaneously.
