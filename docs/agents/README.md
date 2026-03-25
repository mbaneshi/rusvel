# Ready-to-paste agent prompts

Each **`PASTE-*.md`** file is **complete**. Open the file, **Select All**, copy, paste into a **new Cursor Composer chat (Agent mode)**. Do not edit the prompt.

## How many agents at once?

| Run together (low merge conflict) | Files |
|-----------------------------------|--------|
| **Up to 4 in parallel** | `PASTE-01`, `PASTE-02`, `PASTE-08`, `PASTE-09` |
| **Add a 5th** only in a **separate git worktree** or **after** LLM cost merges | `PASTE-10` (touches `rusvel-core` ports) |
| **Exactly one agent on `main`** | `PASTE-06` (`EngineKind` removal — touches whole workspace) |
| **One at a time** (same crates) | `PASTE-11`, `PASTE-12`, `PASTE-14` — each touches `rusvel-agent` or related; do not run two of these together |

**Merge suggestion:** land `PASTE-01` + `PASTE-02` first (depts), then `PASTE-08`, then `PASTE-09`, then `PASTE-10`, then `PASTE-06` last before release.

## Index

| File | Sprint task | Scope |
|------|-------------|--------|
| [PASTE-01-dept-content.md](./PASTE-01-dept-content.md) | #1 | `dept-content`, `content-engine` |
| [PASTE-02-dept-forge.md](./PASTE-02-dept-forge.md) | #2 | `dept-forge`, `forge-engine` |
| [PASTE-08-llm-cost.md](./PASTE-08-llm-cost.md) | #8 | `rusvel-llm`, `rusvel-core` (cost types / routing only) |
| [PASTE-09-approval-ui.md](./PASTE-09-approval-ui.md) | #9 | `frontend` only |
| [PASTE-10-terminal-phase1.md](./PASTE-10-terminal-phase1.md) | #10 | `rusvel-core` + new `rusvel-terminal` crate |
| [PASTE-06-engine-kind.md](./PASTE-06-engine-kind.md) | #6 | **Serial — whole workspace** |
| [PASTE-11-context-compaction.md](./PASTE-11-context-compaction.md) | #11 | `rusvel-agent` |
| [PASTE-12-memory-tools.md](./PASTE-12-memory-tools.md) | #12 | `rusvel-builtin-tools`, `rusvel-agent` |
| [PASTE-14-hybrid-rag.md](./PASTE-14-hybrid-rag.md) | #14 | `rusvel-memory`, `rusvel-vector`, API glue |

Authority: `docs/plans/sprints.md`.
