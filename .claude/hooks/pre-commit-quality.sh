#!/usr/bin/env bash
# Run cargo check on the whole workspace before allowing a git commit hook to proceed.
# Exit 1 blocks the hook; wire in Claude Code PreToolUse or similar for git commit.
set -euo pipefail
cd "$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
exec cargo check --workspace
