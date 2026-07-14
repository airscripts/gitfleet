#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

ENV_NAME="gitfleet-test-env-$GITFLEET_PLAYBOOK_RESOURCE_SUFFIX"
ENV_CREATED=false

setup() { :; }

teardown() {
  if [ "$ENV_CREATED" = true ]; then
    gitfleet environment delete "$ENV_NAME" --repo "$GITFLEET_PLAYBOOK_REPO" --yes >/dev/null 2>&1 || true
  fi
  print_summary
}

trap teardown EXIT
setup

step "List Environments"
expect_exit_0 "environment list succeeds" gitfleet environment list --repo "$GITFLEET_PLAYBOOK_REPO"

step "Create Environment"
if gitfleet environment create "$ENV_NAME" --repo "$GITFLEET_PLAYBOOK_REPO" >/dev/null 2>&1; then
  pass "environment create succeeded"
  ENV_CREATED=true
else
  fail "environment create failed"
fi

step "Create Environment Without --name"
expect_exit_non0 "environment create without name fails" gitfleet environment create --repo "$GITFLEET_PLAYBOOK_REPO"

if [ "$ENV_CREATED" = true ]; then
  step "Delete Environment"
  expect_exit_0 "environment delete succeeds" gitfleet environment delete "$ENV_NAME" --repo "$GITFLEET_PLAYBOOK_REPO" --yes
  ENV_CREATED=false
fi
