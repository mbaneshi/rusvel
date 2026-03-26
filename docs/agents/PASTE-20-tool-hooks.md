# Task #20: PreToolUse / PostToolUse Hooks

> Read this file, then do the task. Only modify files listed below.

## Status: DONE

This task has been completed. The following was implemented:

### domain.rs additions (rusvel-core)
- `HookPoint` enum: `PreToolUse`, `PostToolUse`
- `HookDecision` enum: `Allow`, `Modify(serde_json::Value)`, `Deny(String)`
- `ToolHookConfig` struct: `id`, `hook_point`, `tool_pattern`

### rusvel-agent additions
- `HookHandler` type alias
- `hooks` field on `AgentRuntime` (`RwLock<Vec<(ToolHookConfig, HookHandler)>>`)
- `register_hook()` method
- `match_tool_pattern()` — exact match, prefix* glob, * wildcard
- Pre-hooks wired before `tools.call()` in both `run()` and `run_streaming_loop()`
- Post-hooks wired after `tools.call()` (informational only)

### Files Modified
- `crates/rusvel-core/src/domain.rs`
- `crates/rusvel-agent/src/lib.rs`
