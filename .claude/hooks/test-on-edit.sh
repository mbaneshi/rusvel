#!/usr/bin/env bash
# After edits under crates/*/src/, run cargo test -p <crate> for each affected crate.
set -euo pipefail
root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"
crates=$(jq -r '.. | strings' 2>/dev/null | grep -E '^crates/[^/]+/src/' | sed -n 's|^crates/\([^/]*\)/.*|\1|p' | sort -u)
for c in $crates; do
  [[ -n "$c" ]] || continue
  cargo test -p "$c"
done
exit 0
