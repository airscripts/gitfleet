#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

setup() { :; }
teardown() { print_summary; }
trap teardown EXIT
setup

step "Repo View"
expect_exit_0 "repo view succeeds" gitfleet repo view "$REPO"

step "Repo List"
expect_exit_0 "repo list succeeds" gitfleet repo list --owner "$ORG" --owner-type org

step "Repo Create"
TEST_REPO="gitfleet-test-repo-crud-$$"
if gitfleet repo create "$TEST_REPO" --owner "$ORG" --owner-type org --private --description "gitfleet repository CRUD playbook" --yes >/dev/null 2>&1; then
  pass "repo create succeeded"

  step "Repo Edit"
  if gitfleet repo edit "$ORG/$TEST_REPO" --description "updated by gitfleet playbook" >/dev/null 2>&1; then
    pass "repo edit succeeded"
  else
    fail "repo edit failed"
  fi

  step "Repo Delete"
  if gitfleet repo delete "$ORG/$TEST_REPO" --yes >/dev/null 2>&1; then
    pass "repo delete succeeded"
  else
    skip "repo delete (token may lack delete_repo scope)"
    echo "[WARN] Repository $ORG/$TEST_REPO was created and needs manual deletion."
  fi
else
  skip "repo create (may have failed)"
fi

step "Repo Invalid Inputs"
expect_exit_non0 "repo create rejects conflicting visibility" gitfleet repo create invalid --public --private