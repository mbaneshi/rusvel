#!/usr/bin/env bash
# Warn if edited .rs files are not rustfmt-clean. Never blocks (always exit 0).
set -uo pipefail
while IFS= read -r f; do
  [[ -f "$f" && "$f" == *.rs ]] || continue
  rustfmt --check "$f" 2>/dev/null || echo "WARN: formatting: $f (run cargo fmt)" >&2
done < <(jq -r '.. | strings | select(test("\\.rs$"))' 2>/dev/null)
exit 0
