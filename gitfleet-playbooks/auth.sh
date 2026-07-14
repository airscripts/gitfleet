#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

AUTH_PROFILE="gitfleet-test-auth-$GITFLEET_PLAYBOOK_RESOURCE_SUFFIX"
DETECT_DIR="$GITFLEET_PLAYBOOK_TMPDIR/gitfleet-auth-detect-$GITFLEET_PLAYBOOK_RESOURCE_SUFFIX"
LOGGED_IN=false

setup() { :; }

teardown() {
  if [ "$LOGGED_IN" = true ]; then
    GITFLEET_CREDENTIAL_STORE=file gitfleet auth logout --profile "$AUTH_PROFILE" --yes >/dev/null 2>&1 || true
  fi

  rm -rf "$DETECT_DIR"

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
if GITFLEET_CREDENTIAL_STORE=file gitfleet auth login --profile "$AUTH_PROFILE" <<< "$GITFLEET_PLAYBOOK_TOKEN" >/dev/null 2>&1; then
  pass "auth login succeeded"
  LOGGED_IN=true
else
  skip "auth login (may already be authenticated)"
fi

step "Auth List"
expect_exit_0 "auth list succeeds" gitfleet auth list

step "Auth Detect"
mkdir -p "$DETECT_DIR"
git -C "$DETECT_DIR" init -q
if [ "$GITFLEET_PLAYBOOK_PROVIDER" = "gitlab" ]; then
  git -C "$DETECT_DIR" remote add origin "https://gitlab.com/$GITFLEET_PLAYBOOK_REPO.git"
else
  git -C "$DETECT_DIR" remote add origin "https://github.com/$GITFLEET_PLAYBOOK_REPO.git"
fi
expect_exit_0 "auth detect succeeds" bash -c 'cd "$1" && exec gitfleet auth detect' _ "$DETECT_DIR"

step "Auth Login Without Token"
GITFLEET_CI=true expect_exit_non0 "auth login without token fails" gitfleet auth login

step "Auth Logout"
if [ "$LOGGED_IN" = true ]; then
  expect_exit_0 "auth logout succeeds" env GITFLEET_CREDENTIAL_STORE=file gitfleet auth logout --profile "$AUTH_PROFILE" --yes
  LOGGED_IN=false
else
  skip "auth logout (was not logged in)"
fi
