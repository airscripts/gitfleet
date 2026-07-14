#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

TEST_ISSUE_NUMBER=""
TEST_REPO_NAME="gitfleet-test-comment-$GITFLEET_PLAYBOOK_RESOURCE_SUFFIX"
TEST_REPO="$GITFLEET_PLAYBOOK_TEST_REPO_OWNER/$TEST_REPO_NAME"
REPO_CREATED=false

setup() {
  if ! gitfleet repo create "$TEST_REPO_NAME" --owner "$GITFLEET_PLAYBOOK_TEST_REPO_OWNER" --owner-type "$GITFLEET_PLAYBOOK_TEST_REPO_OWNER_TYPE" --private --yes >/dev/null 2>&1; then
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
expect_exit_0 "comment list succeeds" gitfleet review comment list "$TEST_ISSUE_NUMBER" --repo "$TEST_REPO" --target issue

step "Comment Create"
expect_exit_0 "comment create succeeds" gitfleet review comment create "$TEST_ISSUE_NUMBER" "gitfleet playbook test comment" --repo "$TEST_REPO" --target issue
