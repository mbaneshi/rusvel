You are a RUSVEL implementation agent. Work only inside this repository.

## Task (from docs/plans/sprints.md Sprint 1)

**Task #9 — Approval UI** — `ApprovalQueue.svelte`, inline approve/reject in chat where appropriate, sidebar badge for pending approvals. Backend already exposes `GET /api/approvals`, `POST /api/approvals/{id}/approve`, `POST /api/approvals/{id}/reject` (see `crates/rusvel-api/src/approvals.rs`).

## Read first

1. `CLAUDE.md` (repo root)
2. `docs/plans/sprints.md` (Sprint 1, Track B, task #9)
3. `docs/design/decisions.md` — ADR-008 (approval gates)
4. `frontend/src` — layout, chat, existing components
5. `crates/rusvel-api/src/approvals.rs` — read-only API contract

## Allowed paths

You may edit only:

- `frontend/**`

Do **not** change Rust crates unless you discover a **bug** in the API response shape — then describe it in the report instead of large backend edits.

## Rules

- Use **pnpm** only (`pnpm check`, `pnpm build`).
- Match existing Tailwind / Svelte 5 runes patterns in the repo.

## Deliverables

- Users can see pending approvals, approve/reject from the UI, and see a visible pending count (e.g. sidebar badge).
- `pnpm check` and `pnpm build` succeed.

## Validation (run before ending)

```bash
cd frontend && pnpm check && pnpm build
```

## Report (required — end your reply with this)

```markdown
## Agent report

| Task # | 9 |
| Task title | Approval UI |
| Summary | |

### Files touched
### Commands run
### Blockers
### Ready to merge?
```
