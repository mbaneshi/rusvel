# Ready-to-paste agent prompts

Each **`PASTE-*.md`** file is **complete**. Open the file, **Select All**, copy, paste into a **new** Cursor Composer chat (Agent mode). **Do not edit the prompt.**

Authority: `docs/plans/sprints.md`.

---

## Sprint 1 — completed (reference)

| File | Task |
|------|------|
| [PASTE-01-dept-content.md](./PASTE-01-dept-content.md) | #1 |
| [PASTE-02-dept-forge.md](./PASTE-02-dept-forge.md) | #2 |
| [PASTE-06-engine-kind.md](./PASTE-06-engine-kind.md) | #6 (serial) |
| [PASTE-08-llm-cost.md](./PASTE-08-llm-cost.md) | #8 |
| [PASTE-09-approval-ui.md](./PASTE-09-approval-ui.md) | #9 |
| [PASTE-10-terminal-phase1.md](./PASTE-10-terminal-phase1.md) | #10 |

---

## Advance — Sprint 2 & 3 (parallel packs)

### Pack A — run **up to 3 agents** together (low collision)

| File | Task | Scope |
|------|------|--------|
| [PASTE-13-batch-api.md](./PASTE-13-batch-api.md) | #13 | `rusvel-core` + `rusvel-llm` |
| [PASTE-14-hybrid-rag.md](./PASTE-14-hybrid-rag.md) | #14 | `rusvel-memory`, `rusvel-vector`, API |
| [PASTE-15-frontend-manifest.md](./PASTE-15-frontend-manifest.md) | #15 | `frontend/**` only |

### Pack B — **one at a time** (`rusvel-agent` contention)

| File | Task | Note |
|------|------|------|
| [PASTE-11-context-compaction.md](./PASTE-11-context-compaction.md) | #11 | agent loop only |
| [PASTE-12-memory-tools.md](./PASTE-12-memory-tools.md) | #12 | builtin tools + agent |

Run **11**, merge, then **12** (or separate worktrees).

### Pack C — after **#10** merged + **#15** if it touches layout

| File | Task | Note |
|------|------|------|
| [PASTE-16-terminal-web-bridge.md](./PASTE-16-terminal-web-bridge.md) | #16 | API + frontend `/terminal` |

Prefer merging **#15** before **#16** if both edit navigation.

### Pack D — orchestration (serialize **18+19** if both touch `lib.rs` registration)

| File | Task |
|------|------|
| [PASTE-18-delegate-agent.md](./PASTE-18-delegate-agent.md) | #18 |
| [PASTE-19-invoke-flow.md](./PASTE-19-invoke-flow.md) | #19 |

---

## Quick index (all files)

| File | Sprint | Task |
|------|--------|------|
| PASTE-01 … PASTE-10 | 1 | #1–#10 |
| PASTE-11, PASTE-12 | 2 | #11, #12 |
| PASTE-13 … PASTE-16 | 2 | #13–#16 |
| PASTE-14 | 2 | #14 |
| PASTE-18, PASTE-19 | 3 | #18, #19 |

See also: [COPY-PASTE-BATCH.md](./COPY-PASTE-BATCH.md), [ADVANCE-BATCH.md](./ADVANCE-BATCH.md).
