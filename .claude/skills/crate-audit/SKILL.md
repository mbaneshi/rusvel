---
name: crate-audit
description: Audit a crate for line count, dependency hygiene, and code quality. Use for periodic crate health checks.
allowed-tools: Read, Grep, Glob, Bash
context: fork
agent: Explore
---

Audit the crate at `$ARGUMENTS` (or all crates if no argument given).

Check:
1. **Line count** — each crate should be < 2000 lines of Rust
2. **Dependencies** — flag unused deps, check for duplicate functionality
3. **Public API surface** — are `pub` items intentional or leaking internals?
4. **Error handling** — `thiserror` for libs, `anyhow` for binaries
5. **Test coverage** — does the crate have `#[cfg(test)]` module?
6. **Documentation** — are public traits/types documented?

Output a summary table: crate name, line count, test count, dependency count, issues found.
