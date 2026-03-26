# Repository status

The **canonical** metrics, feature inventory, and gaps for the RUSVEL codebase live in the main repository under `docs/`. This page summarizes what shipped as of the last audit and links to the full Markdown (always current on `main`).

## Canonical sources (GitHub)

| Document | Purpose |
|----------|---------|
| [docs/status/current-state.md](https://github.com/mbaneshi/rusvel/blob/main/docs/status/current-state.md) | **Single written source of truth** — numbers, what works E2E, gaps, re-verify commands |
| [docs/README.md](https://github.com/mbaneshi/rusvel/blob/main/docs/README.md) | Documentation index — which folder is truth vs plans vs scratch |
| [docs/status/verification-log-2026-03-27.md](https://github.com/mbaneshi/rusvel/blob/main/docs/status/verification-log-2026-03-27.md) | Claim → evidence for the metrics snapshot |

When this mdBook page and the repo diverge, **trust the repo files above** and refresh the tables below on the next audit.

---

## Metric definitions (abbrev.)

| Term | Meaning |
|------|---------|
| **Workspace members** | Packages in root `Cargo.toml` `[workspace].members` |
| **HTTP route chains** | Lines with `.route(` in `crates/rusvel-api/src/lib.rs` (one line can register `get().post()`) |
| **API modules** | `*.rs` files in `rusvel-api/src/` except `lib.rs` |

---

## Numbers at a glance (2026-03-27)

| Metric | Count |
|--------|------:|
| Workspace members | 50 |
| Rust LOC (`crates/*.rs`) | ~52,560 |
| Rust source files (`crates/`) | 215 |
| Tests (approx., full `cargo test`) | ~399 |
| Test targets (approx., `cargo test --no-run`) | ~61 |
| HTTP route chains in API router | 105 |
| API modules (`rusvel-api`) | 26 |
| Port traits (`rusvel-core/src/ports.rs`) | 20 |
| Departments / `dept-*` crates | 12 / 13 |
| Engines | 13 (via `DepartmentApp`) |

---

## Gaps (explicit)

- **OutreachSend / GTM jobs** — job worker returns `engine_not_wired` until GTM is integrated.
- **Auth** — not full API middleware; env/in-memory style for many paths.
- **Depth** — Several business engines are thinner than Forge/Code/Content/Harvest; department chat still works via `DepartmentApp`.

---

## How to re-verify

```bash
cargo build
cargo test
cargo metadata --format-version 1 --no-deps | python3 -c "import json,sys; print(len(json.load(sys.stdin)['workspace_members']))"
find crates -name '*.rs' | wc -l
wc -l $(find crates -name '*.rs') | tail -1
rg '\.route\(' crates/rusvel-api/src/lib.rs | wc -l
```

Some tests (e.g. terminal PTY) may fail in restricted environments.
