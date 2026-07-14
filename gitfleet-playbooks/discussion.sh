#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

setup() { :; }

teardown() {
  print_summary
}

trap teardown EXIT
setup

if ! has_capability "discussions"; then
  step "Discussion Capability"
  expect_exit_non0 "discussions are explicitly unsupported" gitfleet discussion list --repo "$GITFLEET_PLAYBOOK_REPO"
  exit 0
fi

step "Discussion List"
if gitfleet discussion list --repo "$GITFLEET_PLAYBOOK_REPO" >/dev/null 2>&1; then
  pass "discussion list succeeded"
else
  skip "discussion list (discussions may not be enabled)"
fi

step "Create Discussion Without Title"
expect_exit_non0 "discussion create without title fails" gitfleet discussion create --repo "$GITFLEET_PLAYBOOK_REPO"
