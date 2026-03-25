# Cursor Agent Playbook — RUSVEL

> **One copy-paste for a new agent:** `docs/plans/NEW_AGENT_PROMPT.md` — read → do → report, nothing else.  
> **Parallel agents / lanes / conflicts:** `docs/plans/AGENT_COORDINATION.md` — read first.  
> **Scheduling source of truth:** `docs/plans/sprints.md` (task numbers **#1–#33**, sprints 1–6).  
> **Deep-dive prompts (B1, F1, …):** `docs/design/agent-workforce.md`  
> **This file:** how to run agents in Cursor, what to paste, how they report, how you review.

---

## 1. Your role (code supervisor)

You do **not** implement every line. You:

1. **Pick one task** from `sprints.md` (one task # per agent chat).
2. **Open a dedicated Cursor Agent / Composer chat** (or a **git worktree** + chat per parallel stream).
3. **Paste** [§3 Base prompt](#3-base-prompt-copy-for-every-agent) + the matching [§5 task block](#5-sprint-1-task-blocks-copy-one) (or a custom block you derive from `sprints.md` for later sprints).
4. **Wait** for the agent’s **§4 Report** before merging.
5. **Review** using [§6 Supervisor checklist](#6-supervisor-review-checklist).
6. **Merge** in dependency order (see `sprints.md` → Critical Path). When in doubt: **ADR-014 / Track A before** things that assume boot + manifests.

**Parallelism:** Safe parallel tasks are those in `sprints.md` marked **parallel** or with **no** “Depends On” to unfinished work. When two agents touch the **same crate** (e.g. `rusvel-agent`), prefer **serial merges** or **split files** explicitly in the task block.

---

## 2. What each agent must read (order)

| Priority | Path | Why |
|----------|------|-----|
| 1 | `docs/plans/sprints.md` | Task scope, exit criteria, dependencies |
| 2 | `CLAUDE.md` (repo root) | Crate layout, ADR rules, commands |
| 3 | Task-specific row in `sprints.md` **Reference Documents** table | Design detail |
| 4 | `docs/design/architecture-v2.md` / `docs/design/decisions.md` | Ports, ADRs |

---

## 3. Base prompt (copy for every agent)

Paste this **first**, then paste one **§5 task block** (or your own task description derived from `sprints.md`).

```markdown
## RUSVEL — Agent session

You are an implementation agent for the RUSVEL monorepo (Rust + SvelteKit).

### Authority
- **Planning / priority:** `docs/plans/sprints.md` is the single source of truth for **what** to build and **task numbers**.
- **Architecture:** Hexagonal — engines and dept crates do **not** import adapter crates; follow `CLAUDE.md` and `docs/design/architecture-v2.md`.

### Your job
1. Read `docs/plans/sprints.md` and locate **Task #…** (given below).
2. Read the **Reference Documents** and ADRs that row depends on (see `sprints.md` table).
3. Implement **only** what that task requires. Do not refactor unrelated code.
4. Respect **allowed paths** in the task block. Do **not** edit files outside the allowlist unless the task explicitly says so.

### Rules
- **Package managers:** `pnpm` only in frontend; `cargo` for Rust; `uv` for Python.
- After changes, run the **Validation** command(s) in the task block and fix failures.
- If you are blocked (missing spec, conflicting ADR), **stop** and report the blocker instead of guessing.

### End of session — REQUIRED output
When finished (or blocked), output **exactly** the report template in `docs/plans/cursor-agent-playbook.md` §4, filled in:
- Task #
- Branch / worktree name (if any)
- Files changed (list)
- Commands run + results
- Ready for merge: yes/no
```

---

## 4. Report template (agent must paste this at the end)

Agents copy this and fill every field:

```markdown
## Agent report — RUSVEL

| Field | Value |
|-------|--------|
| **Task #** | (from sprints.md, e.g. #7) |
| **Task title** | (exact row from sprints.md) |
| **Branch / worktree** | e.g. `agent/task-7-deferred-tools` or `main` |
| **Summary** | 2–4 sentences: what shipped |

### Files touched
- `path/to/file` — …

### Commands run
```
(paste exact commands + exit codes / pass-fail)
```

### Tests / manual checks
- …

### Out of scope (did not do)
- …

### Blockers / follow-ups
- None / …

### Ready for human merge?
**YES** or **NO** — if NO, reason why.

### Risks
- None / …
```

---

## 5. Sprint 1 task blocks (copy one per agent)

Each block is **additional** to [§3 Base prompt](#3-base-prompt-copy-for-every-agent).

### Track A — ADR-014

#### Task #1 — Complete dept-content impl

```markdown
### Task assignment
- **Task #:** 1
- **Title:** Complete dept-content impl (routes, tools, personas, skills in manifest)
- **Read first:** `docs/plans/sprints.md`, `docs/design/department-as-app.md`, `crates/dept-content/`, `crates/content-engine/`

**Allowed paths:** `crates/dept-content/**`, `crates/content-engine/**` (only if required for dept-content API), `crates/rusvel-core/src/department/**` (read/minimal edits if manifest types need extension — prefer avoid; ask if core change is large)

**Forbidden:** unrelated engines, `rusvel-app` boot (that is task #3), broad refactors.

**Validation:** `cargo test -p dept-content` and `cargo test -p content-engine` (if linked)
```

#### Task #2 — Complete dept-forge impl

```markdown
### Task assignment
- **Task #:** 2
- **Title:** Complete dept-forge impl (mission tools, 10 personas, forge routes)
- **Read first:** `docs/plans/sprints.md`, `docs/design/department-as-app.md`, `crates/dept-forge/`, `crates/forge-engine/`

**Allowed paths:** `crates/dept-forge/**`, `crates/forge-engine/**` (minimal), `crates/rusvel-core/src/department/**` (read/minimal)

**Forbidden:** `rusvel-app` boot (task #3).

**Validation:** `cargo test -p dept-forge` and `cargo test -p forge-engine`
```

#### Task #3 — Wire boot sequence

```markdown
### Task assignment
- **Task #:** 3
- **Title:** Wire boot sequence — `main.rs` iterates `INSTALLED_DEPTS`, calls `register()`
- **Read first:** `docs/plans/sprints.md`, `docs/design/department-as-app.md`, `crates/rusvel-app/src/main.rs`, `crates/rusvel-app/src/boot.rs` (if present)

**Allowed paths:** `crates/rusvel-app/**`, `crates/rusvel-core/src/department/**` (if registration API needs tweaks)

**Forbidden:** rewriting all engines in one go (tasks #4–#5 are separate).

**Validation:** `cargo run -- --help` and `cargo test -p rusvel-app` (if tests exist)
```

#### Task #4 — Convert Code, Harvest, Flow → dept-*

```markdown
### Task assignment
- **Task #:** 4
- **Title:** Convert Code, Harvest, Flow engines → dept-* crates
- **Read first:** `docs/plans/sprints.md`, `docs/design/department-as-app.md`, existing `dept-code`, `dept-harvest`, `dept-flow` crates

**Allowed paths:** `crates/dept-code/**`, `crates/dept-harvest/**`, `crates/dept-flow/**`, corresponding `crates/*-engine/**`, `crates/rusvel-core/**` only if required

**Validation:** `cargo test -p dept-code`, `cargo test -p dept-harvest`, `cargo test -p dept-flow`, `cargo check --workspace`
```

#### Task #5 — Convert 8 stub depts → dept-*

```markdown
### Task assignment
- **Task #:** 5
- **Title:** Convert 8 stub depts (finance, growth, etc.) → dept-* crates
- **Read first:** `docs/plans/sprints.md`, `docs/design/department-as-app.md`, existing `crates/dept-*`

**Allowed paths:** `crates/dept-finance/**`, `crates/dept-growth/**`, … (stub depts only), matching `crates/*-engine/**` only as needed

**Validation:** `cargo check --workspace` and `cargo test -p <each crate touched>`
```

#### Task #6 — Remove EngineKind, string IDs

```markdown
### Task assignment
- **Task #:** 6
- **Title:** Remove `EngineKind` enum, use string IDs everywhere
- **Read first:** `docs/plans/sprints.md`, `docs/design/decisions.md` (ADR-005/014), usages of `EngineKind` in repo

**Allowed paths:** wherever `EngineKind` appears — **expect** wide diff; coordinate with supervisor so no other agent touches same files in parallel.

**Validation:** `cargo test --workspace` (or project convention: `cargo test` per CLAUDE.md)
```

### Track B — Quick wins (parallel)

#### Task #7 — Deferred tool loading

```markdown
### Task assignment
- **Task #:** 7
- **Title:** Deferred Tool Loading — `tool_search` meta-tool, split always/searchable
- **Read first:** `docs/plans/sprints.md`, `docs/plans/next-level-proposals.md` (P1), `crates/rusvel-tool/`, `crates/rusvel-builtin-tools/`, `crates/rusvel-agent/`

**Allowed paths:** `crates/rusvel-tool/**`, `crates/rusvel-builtin-tools/**`, `crates/rusvel-agent/**` (as needed)

**Validation:** `cargo test -p rusvel-tool`, `cargo test -p rusvel-builtin-tools`, `cargo test -p rusvel-agent`
```

#### Task #8 — LLM cost intelligence (ModelTier + CostTracker)

```markdown
### Task assignment
- **Task #:** 8
- **Title:** LLM Cost Intelligence — ModelTier routing + CostTracker
- **Read first:** `docs/plans/sprints.md`, `docs/plans/next-level-proposals.md` (P12), `crates/rusvel-llm/`, `crates/rusvel-core/`

**Allowed paths:** `crates/rusvel-llm/**`, `crates/rusvel-core/**` (types/ports only as needed)

**Validation:** `cargo test -p rusvel-llm`, `cargo test -p rusvel-core`
```

#### Task #9 — Approval UI

```markdown
### Task assignment
- **Task #:** 9
- **Title:** Approval UI — ApprovalQueue.svelte, inline approve/reject, sidebar badge
- **Read first:** `docs/plans/sprints.md`, `docs/design/decisions.md` (ADR-008), `frontend/` API usage for approvals

**Allowed paths:** `frontend/**` (exclude `node_modules`), optionally `crates/rusvel-api/**` only if API contract is already wrong

**Validation:** `cd frontend && pnpm check && pnpm build`
```

#### Task #10 — Fix code_to_content tests

```markdown
### Task assignment
- **Task #:** 10
- **Title:** Fix 3 failing code_to_content integration tests
- **Read first:** `docs/plans/sprints.md`, failing test output from `cargo test -p rusvel-api`

**Allowed paths:** `crates/rusvel-api/**`, any code paths tests exercise (minimal)

**Validation:** `cargo test -p rusvel-api` (filter `code_to_content` if available)
```

---

## 6. Supervisor review checklist

Before you merge:

- [ ] **Task #** in the agent report matches `sprints.md`.
- [ ] **Validation commands** were run and match the task block (or stricter).
- [ ] **Allowlist respected** — no surprise edits to unrelated crates.
- [ ] **Architecture:** no new adapter imports from engines/dept crates.
- [ ] **If parallel agents** were used: **merge order** documented; no half-finished `EngineKind` removal spanning branches.
- [ ] **Tests green** on your machine after merge.
- [ ] Update `sprints.md` **Status** column if you track progress there (optional).

---

## 7. Later sprints (tasks #11+)

For **Sprint 2–6**, do **not** duplicate the plan here. Create a new task block by copying §5 structure:

1. Copy **Task #, title, dependencies** from `sprints.md`.
2. Add **Read first** from the **Reference Documents** table in `sprints.md`.
3. Set **Allowed paths** from crate names in the task.
4. Set **Validation** from `CLAUDE.md` + affected crates.

Optional: use `docs/design/agent-workforce.md` agent IDs (F7, B2, …) where they already match the **same** task.

---

## 8. Quick copy: one-liner for lazy agent

```text
Read docs/plans/sprints.md Task #X, docs/design/department-as-app.md if ADR-014, CLAUDE.md; implement only that task; run the validation in cursor-agent-playbook.md §5; end with §4 report.
```
