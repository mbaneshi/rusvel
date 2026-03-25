You are a RUSVEL implementation agent. Work only inside this repository.

## Task (from docs/plans/sprints.md Sprint 2)

**Task #15 — Frontend manifest alignment** — department UI tabs, sidebar routes, and navigation should be driven by `DepartmentManifest` (or `GET` manifest API) instead of hardcoded department lists in Svelte. Remove duplicate hardcoding where safe; keep fallbacks for bootstrapping.

## Read first

1. `CLAUDE.md` (repo root)
2. `docs/plans/sprints.md` (Sprint 2, task #15)
3. `docs/design/department-as-app.md`
4. `frontend/src/routes/+layout.svelte` — nav items
5. `crates/rusvel-core/src/department/manifest.rs` — contribution shapes (read-only)
6. `crates/rusvel-api/` — any existing `GET /api/dept/.../manifest` or departments listing

## Allowed paths

You may edit only:

- `frontend/**`

Do **not** change Rust in this task unless you file a blocker (missing API) — prefer fetching manifest JSON from existing endpoints or add a minimal API change in a **separate** agent if required.

## Rules

- **pnpm** only: `pnpm check && pnpm build`.
- Svelte 5 runes, Tailwind patterns already in repo.

## Validation (run before ending)

```bash
cd frontend && pnpm check && pnpm build
```

## Report (required — end your reply with this)

```markdown
## Agent report

| Task # | 15 |
| Task title | Frontend manifest alignment |
| Summary | |

### Files touched
### Commands run
### Blockers
### Ready to merge?
```
