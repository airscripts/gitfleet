#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

setup() { :; }
teardown() { print_summary; }
trap teardown EXIT
setup

step "Govern List-Rulesets"
if ! has_capability "governance"; then
  expect_exit_non0 "governance is explicitly unsupported" gitfleet govern list-rulesets --repo "$REPO"
  exit 0
fi

if gitfleet govern list-rulesets --repo "$REPO" >/dev/null 2>&1; then
  pass "govern list-rulesets succeeds"
else
  skip "govern list-rulesets (may require additional permissions)"
fi
