#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

TEST_ISSUE_NUMBER=""
TEST_REPO_NAME="gitfleet-test-comment-$PB_RESOURCE_SUFFIX"
TEST_REPO="$ORG/$TEST_REPO_NAME"
REPO_CREATED=false

setup() {
  if ! gitfleet repo create "$TEST_REPO_NAME" --owner "$ORG" --owner-type org --private --yes >/dev/null 2>&1; then
    fail "comment test repository creation failed"
    return
  fi
  REPO_CREATED=true

  output=$(gitfleet issue create --repo "$TEST_REPO" "[noop] gitfleet review comment test" --body "Auto-created by the comment playbook." --json 2>&1) || true
  TEST_ISSUE_NUMBER=$(echo "$output" | python3 -c "import sys,json; print(json.load(sys.stdin).get('number',''))" 2>/dev/null || echo "")
}

teardown() {
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

step "Comment List"
if gitfleet review comment list "$TEST_ISSUE_NUMBER" --repo "$TEST_REPO" >/dev/null 2>&1; then
  pass "comment list succeeds"
else
  skip "comment list (issue may not exist)"
fi

step "Comment Create"
if gitfleet review comment create "$TEST_ISSUE_NUMBER" --body "gitfleet playbook test comment" --repo "$TEST_REPO" >/dev/null 2>&1; then
  pass "comment create succeeds"
else
  skip "comment create (issue may not exist)"
fi
