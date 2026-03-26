# Task #32: Starter Kits

> Read this file, then do the task. Only modify files listed below.

## Goal

One-click department bundles (Indie SaaS, Freelancer, Open Source Maintainer) that seed agents, skills, rules via the existing `!build` + Capability Engine.

## Files to Read First

- `crates/rusvel-api/src/build_cmd.rs` — existing `!build` command handler
- `crates/rusvel-api/src/capability.rs` — Capability Engine
- `crates/rusvel-core/src/domain.rs` — Agent, Skill, Rule types

## What to Build

### 1. Kit types in `crates/rusvel-core/src/domain.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StarterKit {
    pub id: String,
    pub name: String,
    pub description: String,
    pub target_audience: String,       // "indie saas", "freelancer", etc.
    pub departments: Vec<String>,      // which depts this kit configures
    pub entities: Vec<KitEntity>,      // what gets created
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KitEntity {
    pub kind: String,                  // "agent", "skill", "rule", "workflow"
    pub department: String,
    pub name: String,
    pub definition: serde_json::Value, // the full entity payload
}
```

### 2. Built-in kits in `crates/rusvel-api/src/kits.rs` (new)

Define 3 kits as static JSON:
1. **Indie SaaS** — content writer agent, code reviewer agent, growth funnel skills, product feedback rules
2. **Freelancer** — harvest pipeline skills, proposal writer agent, invoice rules, client outreach workflow
3. **Open Source Maintainer** — issue triage agent, release notes skill, community growth rules

### 3. API routes

- `GET /api/kits` — list available kits
- `GET /api/kits/:id` — kit details
- `POST /api/kits/:id/install` — install a kit (creates all entities via StoragePort)

### 4. Wire

- Add `pub mod kits;` to `crates/rusvel-api/src/lib.rs`
- Mount routes

## Files to Modify

- `crates/rusvel-core/src/domain.rs` — add StarterKit, KitEntity
- `crates/rusvel-api/src/kits.rs` (new)
- `crates/rusvel-api/src/lib.rs` — add module + routes

## Verify

```bash
cargo check -p rusvel-api && cargo check --workspace
```
