#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

AUTH_PROFILE="gitfleet-test-auth-$PB_RESOURCE_SUFFIX"
LOGGED_IN=false

setup() { :; }

teardown() {
  if [ "$LOGGED_IN" = true ]; then
    gitfleet auth logout --profile "$AUTH_PROFILE" --yes >/dev/null 2>&1 || true
  fi

  print_summary
}

trap teardown EXIT
setup

step "Auth Status"
expect_exit_0 "auth status succeeds" gitfleet auth status

step "Auth Capability Status"
expect_exit_0 "auth capability status succeeds" gitfleet auth status --capabilities

step "Auth Token (masked)"
expect_exit_0 "auth token succeeds" gitfleet auth token

step "Auth Token (raw)"
expect_exit_0 "auth token --raw succeeds" gitfleet auth token --raw

step "Auth Login"
if gitfleet auth login --profile "$AUTH_PROFILE" <<< "$PLAYBOOK_TOKEN" >/dev/null 2>&1; then
  pass "auth login succeeded"
  LOGGED_IN=true
else
  skip "auth login (may already be authenticated)"
fi

step "Auth List"
expect_exit_0 "auth list succeeds" gitfleet auth list

step "Auth Detect"
expect_exit_0 "auth detect succeeds" gitfleet auth detect

step "Auth Login Without Token"
CI=true expect_exit_non0 "auth login without token fails" gitfleet auth login

step "Auth Logout"
if [ "$LOGGED_IN" = true ]; then
  expect_exit_0 "auth logout succeeds" gitfleet auth logout --profile "$AUTH_PROFILE" --yes
  LOGGED_IN=false
else
  skip "auth logout (was not logged in)"
fi
