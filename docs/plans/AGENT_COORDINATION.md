# Agent coordination — archived snapshot

> **This file is not current.** It described a **2026-03-25** WIP state (e.g. `EngineKind` removal, parallel lanes) that **no longer matches** the repository. **Do not** use it for scheduling or factual claims.
>
> **Use instead:**
> - [`sprints.md`](sprints.md) — task intent and history
> - [`cursor-agent-playbook.md`](cursor-agent-playbook.md) — prompt templates (verify task status before running)
> - [`../status/current-state.md`](../status/current-state.md) — what is wired today (`EngineKind` is removed; **14** `DepartmentApp` departments at boot)

For parallel agents, prefer **non-overlapping paths** and **separate branches/worktrees**; there is no Cursor-wide file lock.
