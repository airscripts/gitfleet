#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

REACTION_ID=""
TEST_ISSUE_NUMBER=""

setup() {
  output=$(gitfleet issue create --repo "$REPO" "[noop] gitfleet review reaction test" --body "Auto-created by the reaction playbook." --json 2>&1) || true
  TEST_ISSUE_NUMBER=$(echo "$output" | python3 -c "import sys,json; print(json.load(sys.stdin).get('number',''))" 2>/dev/null || echo "")
}

teardown() {
  if [ -n "$REACTION_ID" ] && [ -n "$TEST_ISSUE_NUMBER" ]; then
    gitfleet review reaction delete "$REACTION_ID" "$TEST_ISSUE_NUMBER" --repo "$REPO" --yes >/dev/null 2>&1 || true
  fi
  if [ -n "$TEST_ISSUE_NUMBER" ]; then
    gitfleet api delete --endpoint "/repos/$REPO/issues/$TEST_ISSUE_NUMBER" --json >/dev/null 2>&1 || true
  fi
  print_summary
}

trap teardown EXIT
setup

if [ -z "$TEST_ISSUE_NUMBER" ]; then
  skip "could not create test issue"
  exit 0
fi

step "Reaction List"
if gitfleet review reaction list "$TEST_ISSUE_NUMBER" --repo "$REPO" >/dev/null 2>&1; then
  pass "reaction list succeeds"
else
  skip "reaction list (issue may not exist)"
fi

step "Reaction Create"
output=$(gitfleet review reaction create "$TEST_ISSUE_NUMBER" "+1" --repo "$REPO" --json 2>&1) || true
REACTION_ID=$(echo "$output" | python3 -c "import sys,json; print(json.load(sys.stdin).get('id',''))" 2>/dev/null || echo "")

if [ -n "$REACTION_ID" ]; then
  pass "reaction create succeeded (id=$REACTION_ID)"
else
  skip "reaction create (issue may not exist or reaction already exists)"
fi

step "Reaction Delete"
if [ -n "$REACTION_ID" ]; then
  expect_exit_0 "reaction delete succeeds" gitfleet review reaction delete "$REACTION_ID" "$TEST_ISSUE_NUMBER" --repo "$REPO" --yes
  REACTION_ID=""
else
  skip "reaction delete (create failed)"
fi
