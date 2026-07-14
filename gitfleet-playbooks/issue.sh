#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

TEST_ISSUE_NUMBER=""
TEST_REPO_NAME="gitfleet-test-issue-$PB_RESOURCE_SUFFIX"
TEST_REPO="$ORG/$TEST_REPO_NAME"
REPO_CREATED=false

setup() {
  if ! gitfleet repo create "$TEST_REPO_NAME" --owner "$ORG" --owner-type org --private --yes >/dev/null 2>&1; then
    fail "issue test repository creation failed"
    return
  fi
  REPO_CREATED=true

  create_output=$(gitfleet issue create --repo "$TEST_REPO" "[noop] gitfleet playbook test" --body "Auto-created by the gitfleet playbook." --json 2>&1) || true
  TEST_ISSUE_NUMBER=$(echo "$create_output" | python3 -c "import sys,json; print(json.load(sys.stdin).get('number',''))" 2>/dev/null || echo "")

  if [ -n "$TEST_ISSUE_NUMBER" ]; then
    pass "test issue #$TEST_ISSUE_NUMBER created"
  else
    skip "could not create test issue (tests requiring issues will be skipped)"
  fi
}

teardown() {
  if [ "$REPO_CREATED" = true ]; then
    gitfleet repo delete "$TEST_REPO" --yes >/dev/null 2>&1 || true
  fi

  print_summary
}

trap teardown EXIT
setup

step "Issue List"
expect_exit_0 "issue list succeeds" gitfleet issue list --repo "$TEST_REPO" --limit 10

if [ -n "$TEST_ISSUE_NUMBER" ]; then
  step "Issue View"
  expect_exit_0 "issue view succeeds" gitfleet issue view "$TEST_ISSUE_NUMBER" --repo "$TEST_REPO"
else
  skip "issue view (no test issue)"
fi

step "Issue Create Without Title"
expect_exit_non0 "issue create without title fails" gitfleet issue create --repo "$TEST_REPO"
