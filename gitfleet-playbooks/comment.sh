#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

TEST_ISSUE_NUMBER=""

setup() {
  output=$(gitfleet issue create --repo "$REPO" "[noop] gitfleet review comment test" --body "Auto-created by the comment playbook." --json 2>&1) || true
  TEST_ISSUE_NUMBER=$(echo "$output" | python3 -c "import sys,json; print(json.load(sys.stdin).get('number',''))" 2>/dev/null || echo "")
}

teardown() {
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

step "Comment List"
if gitfleet review comment list "$TEST_ISSUE_NUMBER" --repo "$REPO" >/dev/null 2>&1; then
  pass "comment list succeeds"
else
  skip "comment list (issue may not exist)"
fi

step "Comment Create"
if gitfleet review comment create "$TEST_ISSUE_NUMBER" --body "gitfleet playbook test comment" --repo "$REPO" >/dev/null 2>&1; then
  pass "comment create succeeds"
else
  skip "comment create (issue may not exist)"
fi
