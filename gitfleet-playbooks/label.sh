#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

LABEL_NAME="gitfleet-test-label-$GITFLEET_PLAYBOOK_RESOURCE_SUFFIX"
LABEL_CREATED=false

setup() { :; }

teardown() {
  if [ "$LABEL_CREATED" = true ]; then
    gitfleet label delete "$LABEL_NAME" --yes --repo "$GITFLEET_PLAYBOOK_REPO" >/dev/null 2>&1 || true
  fi
  print_summary
}

trap teardown EXIT
setup

step "List Labels"
expect_exit_0 "labels list succeeds" gitfleet label list --repo "$GITFLEET_PLAYBOOK_REPO"

step "Create Label"
if gitfleet label create "$LABEL_NAME" --color ff0000 --description "Test label from gitfleet" --repo "$GITFLEET_PLAYBOOK_REPO" >/dev/null 2>&1; then
  pass "labels create succeeds"
  LABEL_CREATED=true
else
  fail "labels create failed"
fi

step "Delete Label"
expect_exit_0 "labels delete succeeds" gitfleet label delete "$LABEL_NAME" --yes --repo "$GITFLEET_PLAYBOOK_REPO"
LABEL_CREATED=false

step "Delete Label Without --yes"
expect_exit_non0 "labels delete fails without --yes" gitfleet label delete "$LABEL_NAME" --repo "$GITFLEET_PLAYBOOK_REPO"
