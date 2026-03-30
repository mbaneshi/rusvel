#!/usr/bin/env bash
# Fire-and-forget session snapshot to local API; never blocks the hook.
set -uo pipefail
(curl -sS -m 2 -X POST http://localhost:3000/api/system/session-snapshot >/dev/null 2>&1 &) || true
exit 0
