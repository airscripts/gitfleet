#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

SECRET_KEY="GITFLEET_PLAYBOOK_TEST_KEY"
SECRET_VALUE="gitfleet-playbook-test-value"
SECRET_SET=false

setup() { :; }

teardown() {
  if [ "$SECRET_SET" = true ]; then
    step "Deleting Test Secret"
    gitfleet secret delete "$SECRET_KEY" --repo "$REPO" --yes >/dev/null 2>&1 && \
      pass "test secret deleted" || skip "test secret deletion (may already be gone)"
  fi

  print_summary
}

trap teardown EXIT
setup

step "List Secrets"
expect_exit_0 "secret list succeeds" gitfleet secret list --repo "$REPO"

step "Set A Secret"
if gitfleet secret set "$SECRET_KEY" "$SECRET_VALUE" --repo "$REPO" >/dev/null 2>&1; then
  pass "secret set succeeded"
  SECRET_SET=true
else
  fail "secret set failed"
fi

if [ "$SECRET_SET" = true ]; then
  step "List Secrets After Set"
  expect_output "list shows new key" "$SECRET_KEY" gitfleet secret list --repo "$REPO"
else
  skip "secret list after set (secret was not set)"
fi

if [ "$SECRET_SET" = true ]; then
  step "Delete The Secret"
  expect_exit_0 "secret delete succeeds" gitfleet secret delete "$SECRET_KEY" --repo "$REPO" --yes
  SECRET_SET=false
else
  skip "secret delete (secret was not set)"
fi

step "Secret Public-Key"
expect_exit_0 "secret public-key succeeds" gitfleet secret public-key --repo "$REPO"

step "Delete Secret Without Name"
expect_exit_non0 "secret delete without name fails" gitfleet secret delete --repo "$REPO"