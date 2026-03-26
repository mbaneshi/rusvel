# Task #23: Self-Correction Loops

> Read this file, then do the task. Only modify files listed below.

## Status: DONE

This task has been completed. The following was implemented:

### New file: `crates/rusvel-agent/src/verification.rs`
- `VerificationResult` enum: `Pass{confidence}`, `Warn{issues, confidence}`, `Fail{issues, suggested_fix}`
- `VerificationContext` struct: `department_id`, `tool_name`, `original_prompt`
- `VerificationStep` async trait: `name()` + `verify()`
- `VerificationChain` — runs steps in order, short-circuits on first Fail
- `LlmCritiqueStep` — sends output to fast LLM for critique
- `RulesComplianceStep` — checks output against forbidden regex patterns
- 3 unit tests (pass, fail, chain short-circuit)

### Files Modified
- `crates/rusvel-agent/src/verification.rs` (new)
- `crates/rusvel-agent/src/lib.rs` (added pub mod verification + re-exports)
- `crates/rusvel-agent/Cargo.toml` (added regex dependency)
