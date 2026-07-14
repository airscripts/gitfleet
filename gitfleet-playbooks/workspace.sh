#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

WORKSPACE_NAME="gitfleet-test-workspace-$PB_RESOURCE_SUFFIX"
TEST_REPO_NAME="gitfleet-test-workspace-$PB_RESOURCE_SUFFIX"
TEST_REPO="$ORG/$TEST_REPO_NAME"
REPO_CREATED=false
WORKSPACE_CREATED=false

setup() {
  if gitfleet repo create "$TEST_REPO_NAME" --owner "$ORG" --owner-type org --private --yes >/dev/null 2>&1; then
    REPO_CREATED=true
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
    gitfleet workspace remove "$WORKSPACE_NAME" >/dev/null 2>&1 || true
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

step "Restore Archived Repository"
expect_exit_0 "repository unarchive succeeds" gitfleet repo unarchive "$TEST_REPO"

step "Workspace Remove"
expect_exit_0 "workspace remove succeeds" gitfleet workspace remove "$WORKSPACE_NAME"
WORKSPACE_CREATED=false

step "Delete Test Repository"
expect_exit_0 "test repository delete succeeds" gitfleet repo delete "$TEST_REPO" --yes
REPO_CREATED=false
