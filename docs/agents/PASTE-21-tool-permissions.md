# Task #21: Tool Permissions

> Read this file, then do the task. Only modify files listed below.

## Status: DONE

This task has been completed. The following was implemented:

### domain.rs additions (rusvel-core)
- `ToolPermissionMode` enum: `Auto`, `Supervised`, `Locked`
- `ToolPermission` struct: `tool_pattern`, `mode`, `department_id: Option<String>`

### rusvel-tool additions
- `permissions: RwLock<Vec<ToolPermission>>` field on `ToolRegistry`
- `set_permission()` — upserts rule (matches on pattern + dept_id)
- `check_permission(tool_name, dept_id)` — dept-specific > global, exact > prefix > wildcard, default Auto
- Permission check in `call()` — extracts `__department_id` from args, Locked → error, Supervised → "AWAITING_APPROVAL", Auto → proceed

### Files Modified
- `crates/rusvel-core/src/domain.rs`
- `crates/rusvel-tool/src/lib.rs`
