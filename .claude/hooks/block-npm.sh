#!/usr/bin/env bash
# Block tool calls whose payload mentions npm install or npm run (use pnpm in this repo).
set -euo pipefail
body="$(cat)"
if echo "$body" | grep -qE 'npm install|npm run'; then
  echo "Use pnpm, not npm" >&2
  exit 1
fi
exit 0
