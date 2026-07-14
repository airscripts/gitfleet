#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

WORKSPACE_NAME="gitfleet-test-workspace-$GITFLEET_PLAYBOOK_RESOURCE_SUFFIX"
TEST_REPO_NAME="gitfleet-test-workspace-$GITFLEET_PLAYBOOK_RESOURCE_SUFFIX"
TEST_REPO="$GITFLEET_PLAYBOOK_TEST_REPO_OWNER/$TEST_REPO_NAME"
REPO_CREATED=false
WORKSPACE_CREATED=false

wait_for_repo_state() {
  local expected_archived="$1"
  local output

  for _ in {1..10}; do
    output=$(gitfleet repo view "$TEST_REPO" --json 2>/dev/null || true)

    if printf '%s' "$output" | python3 -c 'import json,sys; expected=sys.argv[1] == "true"; data=json.load(sys.stdin); raise SystemExit(data.get("archived") is not expected)' "$expected_archived" 2>/dev/null; then
      return 0
    fi

    sleep 1
  done

  return 1
}

setup() {
  if gitfleet repo create "$TEST_REPO_NAME" --owner "$GITFLEET_PLAYBOOK_TEST_REPO_OWNER" --owner-type "$GITFLEET_PLAYBOOK_TEST_REPO_OWNER_TYPE" --private --yes >/dev/null 2>&1; then
    REPO_CREATED=true

    if ! wait_for_repo_state false; then
      fail "workspace test repository did not become visible"
    fi
  else
    fail "workspace test repository creation failed"
  fi
}

teardown() {
  if [ "$REPO_CREATED" = true ]; then
    gitfleet repo unarchive "$TEST_REPO" >/dev/null 2>&1 || true
    gitfleet repo delete "$TEST_REPO" --yes >/dev/null 2>&1 || true
  fi
  if [ "$WORKSPACE_CREATED" = true ]; then
    gitfleet --yes workspace remove "$WORKSPACE_NAME" >/dev/null 2>&1 || true
  fi
  print_summary
}

trap teardown EXIT
setup

step "Workspace List"
expect_exit_0 "workspace list succeeds" gitfleet workspace list

step "Workspace Define"
if gitfleet workspace define --name "$WORKSPACE_NAME" --repos "$TEST_REPO" >/dev/null 2>&1; then
  pass "workspace define succeeds"
  WORKSPACE_CREATED=true
else
  fail "workspace define failed"
fi

step "Workspace List After Define"
expect_exit_0 "workspace list after define succeeds" gitfleet workspace list

step "Workspace Archive"
expect_exit_0 "workspace archive succeeds" gitfleet workspace archive "$WORKSPACE_NAME" --yes

if ! wait_for_repo_state true; then
  fail "workspace repository did not become archived"
fi

step "Restore Archived Repository"
expect_exit_0 "repository unarchive succeeds" gitfleet repo unarchive "$TEST_REPO"

if ! wait_for_repo_state false; then
  fail "workspace repository did not become unarchived"
fi

step "Workspace Remove"
expect_exit_0 "workspace remove succeeds" gitfleet --yes workspace remove "$WORKSPACE_NAME"
WORKSPACE_CREATED=false

step "Delete Test Repository"
if gitfleet repo delete "$TEST_REPO" --yes >/dev/null 2>&1; then
  pass "test repository delete succeeds"
  REPO_CREATED=false
else
  fail "test repository delete succeeds (exited non-zero)"
fi
