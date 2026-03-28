# Fix Seed Data for New Users

**Status:** Planned
**Priority:** High — blocks any public release
**File:** `crates/rusvel-app/src/main.rs` lines 209-631 (`seed_defaults` function)

## Problem

72% of seeded entities (21 of 29) reference RUSVEL internals — architecture rules, file paths, crate names, design system variables. A new user installing via brew gets an app pre-loaded with developer self-improvement tools that are useless and confusing.

## Current Seed Inventory

### Agents (9 seeded, 7 broken)

| Name | Dept | Issue |
|------|------|-------|
| `rust-engine` | Code | Instructions: "Build **RUSVEL** engine crates. Follow hexagonal architecture." |
| `svelte-ui` | Code | Instructions: "Build SvelteKit 5 pages using **RUSVEL** design system." |
| `test-writer` | Code | Instructions: "Write tests for **RUSVEL** crates and frontend." |
| `content-writer` | Content | OK — generic |
| `proposal-writer` | Harvest | OK — generic |
| `arch-reviewer` | Code | Instructions: "Review code for hexagonal architecture violations" |
| `si-test-writer` | Code | Instructions: "Engine tests use mock ports (never real adapters)" |
| `refactorer` | Code | Instructions: "maintain hexagonal boundaries" |
| `doc-updater` | Code | Instructions: "update docs/status/current-state.md" — hardcoded RUSVEL paths |

### Skills (10 seeded, 5 broken)

| Name | Dept | Issue |
|------|------|-------|
| Code Review | Code | OK |
| Blog Draft | Content | OK but uses `{topic}` not `{{input}}` — inconsistent interpolation |
| Proposal Draft | Harvest | OK |
| Test Generator | Code | Slightly flavored but usable |
| Daily Standup | Forge | OK |
| `/analyze-architecture` | Code | Lists specific RUSVEL engine crate names |
| `/check-crate-sizes` | Code | References "RUSVEL crate sizes" |
| `/enforce-rules` | Code | References RUSVEL design system, hexagonal boundaries |
| `/self-review` | Code | Hardcodes `docs/status/gap-analysis.md` path |
| `/update-docs` | Code | Hardcodes `docs/status/current-state.md` path |

### Rules (9 seeded, 8 broken)

| Name | Dept | Issue |
|------|------|-------|
| Hexagonal Architecture | Code | RUSVEL architecture rule |
| Human Approval Gate | Global | OK — good universal default |
| Crate Size Limit | Code | RUSVEL-specific limit |
| Hexagonal Boundaries (SI) | Code | Duplicate of #1 |
| Crate Size Limit (SI) | Code | Duplicate of #3 |
| Event Sourcing (SI) | Code | References EventPort — RUSVEL internal |
| Metadata Evolution (SI) | Code | References ADR-007 — RUSVEL internal |
| Test Coverage (SI) | Code | "Engine tests use mock ports" — RUSVEL pattern |
| Design Tokens (SI) | Code | "--r-* CSS variables" — RUSVEL design system |

### Workflows (1 seeded, 1 broken)

| Name | Issue |
|------|-------|
| `self-improve` | 4-step RUSVEL self-improvement cycle referencing internal agents |

### Profile Wizard (line 660)

| Field | Issue |
|-------|-------|
| Name placeholder | `"e.g. Mehdi"` — developer's personal name |

## Fix Plan

### 1. Split into user seeds vs dev seeds

```rust
async fn seed_defaults(storage: &Arc<dyn StoragePort>) -> Result<()> {
    seed_user_defaults(storage).await?;   // Always run
    Ok(())
}

// Only called with `rusvel --seed-dev` or env RUSVEL_SEED_DEV=1
async fn seed_dev_defaults(storage: &Arc<dyn StoragePort>) -> Result<()> {
    seed_self_improvement_agents(storage).await?;
    seed_self_improvement_skills(storage).await?;
    seed_self_improvement_rules(storage).await?;
    seed_self_improvement_workflow(storage).await?;
    Ok(())
}
```

### 2. User defaults (what every new user should get)

**Agents (5):**
- `backend-engineer` — "Assist with backend development in your project"
- `frontend-engineer` — "Assist with frontend development and UI"
- `content-writer` — "Draft blog posts, articles, and long-form content" (keep as-is)
- `proposal-writer` — "Draft compelling proposals for opportunities" (keep as-is)
- `researcher` — "Research topics, competitors, and market trends"

**Skills (5):**
- `Code Review` — keep, it's generic
- `Blog Draft` — fix interpolation to use `{{input}}`
- `Proposal Draft` — keep
- `Daily Standup` — keep
- `Test Generator` — keep, remove "Rust module or SvelteKit" specificity

**Rules (2):**
- `Human Approval Gate` — keep (universal safety)
- `Be Concise` — "Keep responses focused and actionable. Avoid filler."

**Workflows (1):**
- `content-pipeline` — "Draft → Review → Publish" generic content workflow

### 3. Dev defaults (opt-in for RUSVEL developers)

Move all self-improvement entities (agents, skills, rules, workflow) behind a flag:
- CLI flag: `rusvel --seed-dev`
- Env var: `RUSVEL_SEED_DEV=1`
- Or: a "Developer Mode" toggle in settings UI

### 4. Fix wizard placeholder

Line 660: Change `"e.g. Mehdi"` → `"Your name"`

### 5. Fix skill interpolation

Blog Draft template uses `{topic}`, `{points}`, `{audience}` but the skill resolver uses `{{input}}`. Either:
- Standardize all templates to `{{input}}` (simpler)
- Or support named parameters in resolve_skill (more complex)

## Files to Modify

- `crates/rusvel-app/src/main.rs` — `seed_defaults()` function (lines 209-631), `first_run_wizard()` (line 660)
- `crates/rusvel-cli/src/lib.rs` — add `--seed-dev` flag to Cli struct

## Verification

```bash
# Delete existing data and re-run
rm -rf ~/.rusvel
cargo run
# Verify: only generic agents/skills/rules appear
# Verify: no RUSVEL-specific references in seeded data
# Verify: wizard placeholder says "Your name" not "Mehdi"

# Test dev seed flag
rm -rf ~/.rusvel
RUSVEL_SEED_DEV=1 cargo run
# Verify: self-improvement entities also appear
```
