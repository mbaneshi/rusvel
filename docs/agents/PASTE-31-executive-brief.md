# Task #31: Executive Brief

> Read this file, then do the task. Only modify files listed below.

## Goal

Enhanced `forge mission today` that delegates to each department and produces a cross-dept daily digest.

## Files to Read First

- `crates/forge-engine/src/lib.rs` — ForgeEngine, Mission, generate_daily_plan
- `crates/rusvel-builtin-tools/src/delegate.rs` — delegate_agent tool
- `crates/rusvel-agent/src/persona.rs` — PersonaCatalog, available personas
- `crates/rusvel-api/src/engine_routes.rs` — existing engine API routes

## What to Build

### 1. Brief types in `crates/rusvel-core/src/domain.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutiveBrief {
    pub id: String,
    pub date: chrono::NaiveDate,
    pub sections: Vec<BriefSection>,
    pub summary: String,              // 2-3 sentence executive summary
    pub action_items: Vec<String>,    // top 3-5 things needing attention
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BriefSection {
    pub department: String,
    pub status: String,               // "green", "yellow", "red"
    pub highlights: Vec<String>,      // 2-3 bullet points
    pub metrics: serde_json::Value,   // dept-specific KPIs
}
```

### 2. Brief generation in `crates/forge-engine/src/lib.rs`

Add `pub async fn generate_brief(&self) -> Result<ExecutiveBrief>`:
1. For each active department, delegate to an agent with that dept's persona asking "What happened today? Key metrics? What needs attention?"
2. Collect all responses into BriefSections
3. Send all sections to a "strategist" agent to produce the summary + action items
4. Return the complete ExecutiveBrief

### 3. API + CLI

- `GET /api/brief` and `POST /api/brief/generate` in `crates/rusvel-api/src/engine_routes.rs`
- `rusvel brief` CLI command in `crates/rusvel-cli/src/lib.rs`

## Files to Modify

- `crates/rusvel-core/src/domain.rs` — add ExecutiveBrief, BriefSection
- `crates/forge-engine/src/lib.rs` — add generate_brief
- `crates/rusvel-api/src/engine_routes.rs` — add brief routes
- `crates/rusvel-cli/src/lib.rs` — add brief command

## Verify

```bash
cargo check -p forge-engine && cargo check -p rusvel-api && cargo check --workspace
```

## Depends On

- #18 delegate_agent (done)
