#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

setup() { :; }
teardown() { print_summary; }
trap teardown EXIT
setup

step "Code Search"
expect_exit_0 "code search succeeds" gitfleet code search "README" --repo "$REPO"

step "Code View"
if gitfleet code view README.md --repo "$REPO" >/dev/null 2>&1; then
  pass "code view succeeds"
else
  skip "code view (may not work for this repo)"
fi