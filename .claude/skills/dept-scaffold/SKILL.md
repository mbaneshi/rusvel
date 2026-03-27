---
name: dept-scaffold
description: Scaffold a new department crate following ADR-014 DepartmentApp pattern. Use when creating new departments.
allowed-tools: Read, Glob, Grep, Bash, Write, Edit
---

Scaffold a new department named `$ARGUMENTS`.

Follow the existing pattern by reading a working example:
1. Read `crates/dept-forge/` as the reference implementation
2. Read `crates/forge-engine/` for the engine pattern

Create these files:
1. `crates/$ARGUMENTS-engine/Cargo.toml` — depend only on `rusvel-core`
2. `crates/$ARGUMENTS-engine/src/lib.rs` — engine with domain logic skeleton
3. `crates/dept-$ARGUMENTS/Cargo.toml` — depend on engine + `rusvel-core`
4. `crates/dept-$ARGUMENTS/src/lib.rs` — implement `DepartmentApp` trait + `DepartmentManifest`

Then update:
5. Root `Cargo.toml` — add both crates to workspace members
6. `crates/rusvel-app/Cargo.toml` — add `dept-$ARGUMENTS` dependency
7. `crates/rusvel-app/src/main.rs` — register department in boot sequence

Keep each crate minimal. Use `thiserror` for errors. Add `metadata: serde_json::Value` to any domain types.
