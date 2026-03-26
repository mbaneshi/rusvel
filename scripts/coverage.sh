#!/usr/bin/env bash
# Workspace LLVM coverage. Requires: cargo install cargo-llvm-cov, rustup component add llvm-tools-preview
set -euo pipefail
cd "$(dirname "$0")/.."
if [[ $# -eq 0 ]]; then
  exec cargo llvm-cov test --workspace --html --summary-only
else
  exec cargo llvm-cov test --workspace "$@"
fi
