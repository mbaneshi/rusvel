# Agent coordination — single entry point

> **Read this file first.** Then follow **your lane only**.  
> Scheduling authority: `docs/plans/sprints.md`  
> Prompt templates + report format: `docs/plans/cursor-agent-playbook.md`  
> Last assessed: **2026-03-25** (repo snapshot below)

---

## 1. Assessment — latest work (what exists now)

### 1.1 Git / branch state

- Branch **`main`**, **ahead of `origin/main`** (local commits include playbook + prior feature work).
- **Large uncommitted working tree**: ADR-014–related changes span many crates (`rusvel-core`, all `dept-*`, `rusvel-app`, engines, `rusvel-agent`, `rusvel-builtin-tools`, API tests, `frontend/flows`, etc.).
- **Implication:** There is **no second human** “on the branch” — but **any new parallel agent** that edits the same files as this WIP will **merge-conflict**. Treat the current diff as **one integration batch** unless you split by **lane** (below).

### 1.2 What already looks implemented (WIP on disk)

| Area | Status | Notes |
|------|--------|--------|
| `crates/rusvel-core/src/department/` | Present (untracked) | `DepartmentApp`, manifest, registration context |
| `crates/rusvel-app/src/boot.rs` | Present | `installed_departments()` lists **12** depts; `boot_departments()` validates + registers |
| `crates/rusvel-app/src/main.rs` | Modified | Calls `boot::installed_departments()` + `boot::boot_departments(...)` |
| All **`crates/dept-*`** (12) | Present (untracked) | Pattern: `*Department` + `register()` + `manifest`; depth varies |
| `dept-content` / `dept-forge` | Substantial | Content registers tools/platforms; Forge registers engine but **may still need** full mission tools / personas / routes per `sprints.md` #1–#2 |
| `crates/rusvel-builtin-tools/src/tool_search.rs` | Present | **Deferred tool loading** (Task #7) started |
| `EngineKind` | Still in codebase | Task #6 **not** done — enum removal remains a **single-lane** job later |
| `docs/plans/sprints.md` | Says “boot not started”, “3 failing tests” | **May be stale** relative to disk — re-verify tests locally after merge |

### 1.3 “What other agent is busy?”

- **Cursor does not reserve files.** “Busy” means: **another chat/worktree is editing the same paths**.
- **Right now:** the **entire ADR-014 surface** is in flux in **one** working tree → **do not** spawn multiple agents on **`rusvel-app`**, **`rusvel-agent`**, **`main.rs`**, or **all depts at once** until you **commit or split work** by lane.

---

## 2. Systematic approach (order of operations)

### Phase 0 — Integration gate (do this before parallel lanes)

**Goal:** One coherent baseline: compiles, tests you care about pass, WIP committed or branched.

| Step | Owner | Action |
|------|--------|--------|
| P0.1 | You | **Commit or branch** current WIP (e.g. `feat/adr-014-wip`) so parallel work can rebase cleanly |
| P0.2 | One agent | `cargo test` / `pnpm check` on that branch — fix **blockers** (`sprints.md` #10 if still red) |
| P0.3 | You | Update **`sprints.md` “Where We Are”** to match reality (boot wired? tests?) |

**Until P0 is done:** use **at most one** implementation agent on the repo, or **separate git worktrees** per agent with **non-overlapping branches**.

### Phase 1 — Parallel lanes (after Phase 0)

Only **one agent per lane**. Lanes are **disjoint by path** to minimize conflicts.

| Lane | Task # (`sprints.md`) | Owned paths (edit **only** these) | Do **not** touch |
|------|------------------------|-----------------------------------|------------------|
| **L1** | #9 Approval UI | `frontend/src/**` (except shared if someone else owns) | Rust crates |
| **L2** | #8 Cost / ModelTier | `crates/rusvel-llm/**`, `crates/rusvel-core/src/ports.rs` (or minimal new types file) | `rusvel-agent`, `dept-*` |
| **L3** | #10 Fix API tests | `crates/rusvel-api/tests/**`, minimal fixes in `rusvel-api/src/**` only | `dept-*`, `boot.rs`, engines |
| **L4** | #7 Deferred tools (finish) | `crates/rusvel-tool/**`, `crates/rusvel-builtin-tools/**`, `crates/rusvel-agent/**` | `dept-*`, `rusvel-app`, `core/department` |
| **L5** | #1 dept-content polish | `crates/dept-content/**`, `crates/content-engine/**` | `dept-forge`, `rusvel-app`, `boot.rs` |
| **L6** | #2 dept-forge polish | `crates/dept-forge/**`, `crates/forge-engine/**` | `dept-content`, `rusvel-app`, `boot.rs` |
| **L7** | #4 / #5 dept migrations (if still needed) | **One dept crate per agent** e.g. only `dept-gtm/**` + `gtm-engine/**` | Other `dept-*` |

**Forbidden parallel combo:** **L4 + any other lane touching `rusvel-agent`** (e.g. L4 + L5 both need agent — **serial**).

### Phase 2 — Serial (one agent, or strict merge order)

| Task # | Work | Why serial |
|--------|------|------------|
| #3 | Boot / `main.rs` / `boot.rs` wiring tweaks | Touches composition root — merge after dept crates stable |
| #6 | Remove `EngineKind` | **Global** rename — **no** parallel work in `rusvel-core` + consumers |

---

## 3. Instructions for each agent (copy-paste)

**Every agent session:**

1. Open **`docs/plans/cursor-agent-playbook.md`** → paste **§3 Base prompt**.
2. Open **`docs/plans/sprints.md`** → find **Task #** from your lane (§2 table).
3. Paste **only** the task block for that task from **cursor-agent-playbook §5** (or a one-lane variant below).
4. Add this line at the top:

```text
Lane: L# (from AGENT_COORDINATION.md §2). Do not edit files outside the lane allowlist.
```

5. End with **§4 Report** in `cursor-agent-playbook.md`.

### One-lane variants (if playbook §5 is too broad)

- **L1 only:** “Implement Task #9; allowed paths `frontend/src/**`; run `pnpm check && pnpm build`.”
- **L2 only:** “Implement Task #8; allowed paths `crates/rusvel-llm/**` + minimal `rusvel-core` types; run `cargo test -p rusvel-llm`.”
- **L3 only:** “Implement Task #10; allowed paths `crates/rusvel-api/tests/**` + minimal `rusvel-api/src` fixes; run `cargo test -p rusvel-api`.”

---

## 4. Supervisor — how you check (no merge surprises)

1. **Lane allowlist** matches diff (no stray `rusvel-app` edits on L1).
2. **Report** includes commands + pass/fail.
3. **Conflict scan:** two PRs touching `rusvel-agent`? → **reorder merges** or **drop one lane** to serial.
4. After all lanes: **one** integration agent or you run **Phase 2** (#6 last).

---

## 5. Conflict matrix (quick reference)

```
        L1   L2   L3   L4   L5   L6   L7
L1      -    ok   ok   ok   ok   ok   ok
L2      ok   -    ok   ok   ok   ok   ok
L3      ok   ok   -    ok   ok   ok   ok
L4      ok   ok   ok   -   conflict conflict ok*
L5      ok   ok   ok  conflict -   ok   ok
L6      ok   ok   ok  conflict ok   -   ok
L7*     ok   ok   ok   ok   ok   ok   ok** 

* L7 = one dept per agent — ok vs ok only if different dept crates.
** conflict if L7 touches same dept as L5/L6.
```

`conflict` with L4 = **L4 edits `rusvel-agent`**; L5/L6 may touch tool registration paths — **verify** before parallel.

---

## 6. Summary

| Question | Answer |
|----------|--------|
| How many agents at once? | After Phase 0: **up to ~5** if lanes L1–L3 + L5 + L6 **and** L4 not running; **L4 alone** or **after** L5/L6 if agent loop conflicts. |
| Prerequisite | **Commit/branch WIP** + green baseline **or** **git worktrees** per lane. |
| Single file for agents | **This file** (`AGENT_COORDINATION.md`) + **cursor-agent-playbook.md** for prompts/reports. |
