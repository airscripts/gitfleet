#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

TEST_DISCUSSION_NUMBER=""

setup() { :; }

teardown() {
  print_summary
}

trap teardown EXIT
setup

step "Discussion List"
if gitfleet discussion list --repo "$REPO" >/dev/null 2>&1; then
  pass "discussion list succeeded"
else
  skip "discussion list (discussions may not be enabled)"
fi

step "Create Discussion"
if gitfleet discussion create --repo "$REPO" --title "[noop] gitfleet test discussion" --body "gitfleet playbook test" >/dev/null 2>&1; then
  pass "discussion create succeeded"
else
  skip "discussion create (discussions may not be enabled)"
fi

step "Create Discussion Without Title"
expect_exit_non0 "discussion create without title fails" gitfleet discussion create --repo "$REPO"