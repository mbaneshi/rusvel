# New agent — one copy-paste

**You:** Fill the `FILL:` lines, then copy **from** `---START---` **through** `---END---` into a **new** Cursor Composer chat (Agent mode).

---

---START---

You work in the RUSVEL repo (Rust + SvelteKit). Follow **CLAUDE.md** at the repo root for commands and ADRs.

## Read (do this first, in order)

1. `CLAUDE.md`
2. `docs/plans/sprints.md` — find the row for **Task #FILL:number** below
3. Any doc linked from that task’s “Reference Documents” in `sprints.md` (or `docs/design/department-as-app.md` if the task is ADR-014)

## Task

- **Task #:** FILL:number (e.g. 8)
- **What to deliver:** FILL:one line (copy the task title from `sprints.md`)

Implement **only** this task. Do not refactor unrelated code. Respect hexagonal rules: engines do not import adapter crates (`CLAUDE.md`).

## Do

- Use **pnpm** in `frontend/`, **cargo** for Rust, **uv** for Python scripts.
- Run the tests/checks that match what you changed (e.g. `cargo test -p <crate>`, `cd frontend && pnpm check`).

## Report (required — end your reply with this, filled in)

    ## Agent report
    | Task # | … |
    | Task title | … |
    | Summary | 2–4 sentences |
    ### Files touched
    - path — what changed
    ### Commands run
    (exact commands and pass/fail)
    ### Blockers
    None / …
    ### Ready to merge?
    YES or NO

---END---

---

**Optional:** If two agents run at once, give each different **folders only** so they do not edit the same files (see `docs/plans/AGENT_COORDINATION.md`).
