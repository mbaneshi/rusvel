
## Overview

The Code department handles all software development tasks. Its agent has access to Claude Code tools -- it can read, write, and edit files, run shell commands, search codebases, and manage background tasks. Currently optimized for Rust with plans to expand to other languages.

Powered by the Code engine, which provides tree-sitter parsing, symbol graphs, BM25 search, and complexity metrics.

## Quick Actions

| Action | What It Does |
|--------|-------------|
| **Analyze codebase** | Structure, dependencies, and code quality report |
| **Run tests** | Execute the full test suite and report results |
| **Find TODOs** | Locate all TODO, FIXME, and HACK comments |

## Example Prompts

- "Analyze the codebase structure, dependencies, and code quality."
- "Implement a new endpoint for user authentication."
- "Refactor the storage module to reduce duplication."
- "Run `cargo test` and fix any failing tests."
- "Find all functions longer than 50 lines and suggest splits."

## Configuration

The Code department has special defaults:

- **Effort:** `high` -- for thorough code analysis
- **Permission mode:** `default` -- allows file operations and shell commands
- **Working directories:** configurable via the Dirs tab

## Tabs

Actions, Agents, Workflows, Skills, Rules, MCP, Hooks, Dirs, Events

## Seeded Agents

| Agent | Role |
|-------|------|
| **rust-engine** | Rust development, writes and refactors Rust code |
| **svelte-ui** | SvelteKit frontend components and styling |
| **test-writer** | Writes unit and integration tests |
