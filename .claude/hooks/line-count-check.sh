#!/usr/bin/env bash
# Warn if any edited crate's src tree exceeds 2000 lines (workspace guideline).
set -uo pipefail
root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
crates=$(jq -r '.. | strings | select(test("\\.rs$"))' 2>/dev/null | sed -n 's|^crates/\([^/]*\)/.*|\1|p' | sort -u)
for c in $crates; do
  [[ -n "$c" ]] || continue
  d="$root/crates/$c/src"
  [[ -d "$d" ]] || continue
  n=$(find "$d" -name '*.rs' -exec cat {} + 2>/dev/null | wc -l | tr -d ' ')
  [[ "${n:-0}" -gt 2000 ]] && echo "WARN: crate $c has ${n} lines under src/ (>2000)" >&2
done
exit 0
