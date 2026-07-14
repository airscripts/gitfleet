#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

TEST_REPO="gitfleet-test-repo-$GITFLEET_PLAYBOOK_RESOURCE_SUFFIX"
TEST_REPO_PATH="$GITFLEET_PLAYBOOK_TEST_REPO_OWNER/$TEST_REPO"
REPO_CREATED=false
setup() { :; }
teardown() {
  if [ "$REPO_CREATED" = true ]; then
    gitfleet repo delete "$TEST_REPO_PATH" --yes >/dev/null 2>&1 || true
  fi
  print_summary
}
trap teardown EXIT
setup

step "Repo View"
expect_exit_0 "repo view succeeds" gitfleet repo view "$GITFLEET_PLAYBOOK_REPO"

step "Repo List"
expect_exit_0 "repo list succeeds" gitfleet repo list --owner "$GITFLEET_PLAYBOOK_ORG" --owner-type org

step "Fork List"
expect_exit_0 "repo fork list succeeds" gitfleet repo fork list --repo "$GITFLEET_PLAYBOOK_REPO"

step "Repo Create"
if gitfleet repo create "$TEST_REPO" --owner "$GITFLEET_PLAYBOOK_TEST_REPO_OWNER" --owner-type "$GITFLEET_PLAYBOOK_TEST_REPO_OWNER_TYPE" --private --description "gitfleet repository CRUD playbook" --yes >/dev/null 2>&1; then
  pass "repo create succeeded"
  REPO_CREATED=true

  step "Repo Edit"
  if gitfleet repo edit "$TEST_REPO_PATH" --description "updated by gitfleet playbook" >/dev/null 2>&1; then
    pass "repo edit succeeded"
  else
    fail "repo edit failed"
  fi

  step "Repo Delete"
  if gitfleet repo delete "$TEST_REPO_PATH" --yes >/dev/null 2>&1; then
    pass "repo delete succeeded"
    REPO_CREATED=false
  else
    fail "repo delete failed"
  fi
else
  fail "repo create failed"
fi

step "Repo Invalid Inputs"
expect_exit_non0 "repo create rejects conflicting visibility" gitfleet repo create invalid --public --private
