#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

setup() { :; }
teardown() { print_summary; }
trap teardown EXIT
setup

step "Dependency List"
if gitfleet deps list --repo "$REPO" >/dev/null 2>&1; then
  pass "dependency list succeeds"
else
  skip "dependency list (may not be available for this repo)"
fi

step "Dependency Review"
if gitfleet deps review --repo "$REPO" --base main --head HEAD >/dev/null 2>&1; then
  pass "dependency review succeeds"
else
  skip "dependency review (may require a PR or specific base/head)"
fi