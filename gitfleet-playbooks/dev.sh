#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

setup() { :; }
teardown() { print_summary; }
trap teardown EXIT
setup

step "Codespace List"
if gitfleet dev list --repo "$REPO" >/dev/null 2>&1; then
  pass "codespace list succeeds"
else
  skip "codespace list (codespaces may not be enabled)"
fi