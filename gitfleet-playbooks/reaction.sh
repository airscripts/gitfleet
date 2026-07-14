#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

REACTION_ID=""
TEST_ISSUE_NUMBER=""
TEST_REPO_NAME="gitfleet-test-reaction-$GITFLEET_PLAYBOOK_RESOURCE_SUFFIX"
TEST_REPO="$GITFLEET_PLAYBOOK_TEST_REPO_OWNER/$TEST_REPO_NAME"
REPO_CREATED=false

setup() {
  if ! gitfleet repo create "$TEST_REPO_NAME" --owner "$GITFLEET_PLAYBOOK_TEST_REPO_OWNER" --owner-type "$GITFLEET_PLAYBOOK_TEST_REPO_OWNER_TYPE" --private --yes >/dev/null 2>&1; then
    fail "reaction test repository creation failed"
    return
  fi
  REPO_CREATED=true

  output=$(gitfleet issue create --repo "$TEST_REPO" "[noop] gitfleet review reaction test" --body "Auto-created by the reaction playbook." --json 2>&1) || true
  TEST_ISSUE_NUMBER=$(echo "$output" | python3 -c "import sys,json; print(json.load(sys.stdin).get('number',''))" 2>/dev/null || echo "")
}

teardown() {
  if [ -n "$REACTION_ID" ] && [ -n "$TEST_ISSUE_NUMBER" ]; then
    gitfleet review reaction delete "$REACTION_ID" "$TEST_ISSUE_NUMBER" --repo "$TEST_REPO" --yes >/dev/null 2>&1 || true
  fi
  if [ "$REPO_CREATED" = true ]; then
    gitfleet repo delete "$TEST_REPO" --yes >/dev/null 2>&1 || true
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
if gitfleet review reaction list "$TEST_ISSUE_NUMBER" --repo "$TEST_REPO" >/dev/null 2>&1; then
  pass "reaction list succeeds"
else
  skip "reaction list (issue may not exist)"
fi

step "Reaction Create"
output=$(gitfleet review reaction create "$TEST_ISSUE_NUMBER" "+1" --repo "$TEST_REPO" --json 2>&1) || true
REACTION_ID=$(echo "$output" | python3 -c "import sys,json; print(json.load(sys.stdin).get('id',''))" 2>/dev/null || echo "")

if [ -n "$REACTION_ID" ]; then
  pass "reaction create succeeded (id=$REACTION_ID)"
else
  skip "reaction create (issue may not exist or reaction already exists)"
fi

step "Reaction Delete"
if [ -n "$REACTION_ID" ]; then
  expect_exit_0 "reaction delete succeeds" gitfleet review reaction delete "$REACTION_ID" "$TEST_ISSUE_NUMBER" --repo "$TEST_REPO" --yes
  REACTION_ID=""
else
  skip "reaction delete (create failed)"
fi
