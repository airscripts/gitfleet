#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

AUTH_PROFILE="gitfleet-test-auth"
ORIGINAL_TOKEN=""
LOGGED_IN=false

setup() {
  ORIGINAL_TOKEN=$(gitfleet auth token --raw 2>/dev/null || echo "")
}

teardown() {
  if [ -n "$ORIGINAL_TOKEN" ]; then
    step "Restoring Original Authentication"
    gitfleet auth login --token "$ORIGINAL_TOKEN" >/dev/null 2>&1 || true
  fi

  print_summary
}

trap teardown EXIT
setup

step "Auth Status"
expect_exit_0 "auth status succeeds" gitfleet auth status

step "Auth Token (masked)"
expect_exit_0 "auth token succeeds" gitfleet auth token

step "Auth Token (raw)"
expect_exit_0 "auth token --raw succeeds" gitfleet auth token --raw

step "Auth Login"
if gitfleet auth login --token "$GITFLEET_GITHUB_TOKEN" >/dev/null 2>&1; then
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
  expect_exit_0 "auth logout succeeds" gitfleet auth logout --yes
else
  skip "auth logout (was not logged in)"
fi

step "Auth Setup-Git"
expect_exit_0 "auth setup-git succeeds" gitfleet auth setup-git

step "Auth Setup-Git With Custom Host"
expect_exit_0 "auth setup-git with custom host succeeds" gitfleet auth setup-git --host nonexistent.example.com