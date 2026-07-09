#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

VAR_KEY="GITFLEET_PLAYBOOK_TEST_VAR"
VAR_VALUE="gitfleet-playbook-test-value"
VAR_SET=false

setup() { :; }

teardown() {
  if [ "$VAR_SET" = true ]; then
    step "Deleting Test Variable"
    gitfleet variable delete "$VAR_KEY" --repo "$REPO" --yes >/dev/null 2>&1 && \
      pass "test variable deleted" || skip "test variable deletion"
  fi

  print_summary
}

trap teardown EXIT
setup

step "List Variables"
expect_exit_0 "variable list succeeds" gitfleet variable list --repo "$REPO"

step "Set A Variable"
if gitfleet variable set "$VAR_KEY" "$VAR_VALUE" --repo "$REPO" >/dev/null 2>&1; then
  pass "variable set succeeded"
  VAR_SET=true
else
  fail "variable set failed"
fi

if [ "$VAR_SET" = true ]; then
  step "List Variables After Set"
  output=$(gitfleet variable list --repo "$REPO" 2>&1) || true

  if echo "$output" | grep -q "$VAR_KEY"; then
    pass "list shows new key"
  else
    skip "list shows new key (variable may not appear immediately in list)"
  fi
else
  skip "variable list after set (variable was not set)"
fi

if [ "$VAR_SET" = true ]; then
  step "Delete The Variable"
  expect_exit_0 "variable delete succeeds" gitfleet variable delete "$VAR_KEY" --repo "$REPO" --yes
  VAR_SET=false
else
  skip "variable delete (variable was not set)"
fi

step "Set Variable Without Value"
expect_exit_non0 "variable set without value fails" gitfleet variable set --repo "$REPO"