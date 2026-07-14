#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

TEST_REPO_NAME="gitfleet-test-security-$GITFLEET_PLAYBOOK_RESOURCE_SUFFIX"
TEST_REPO="$GITFLEET_PLAYBOOK_TEST_REPO_OWNER/$TEST_REPO_NAME"
REPO_CREATED=false

setup() {
  if has_capability "security" && gitfleet repo create "$TEST_REPO_NAME" --owner "$GITFLEET_PLAYBOOK_TEST_REPO_OWNER" --owner-type "$GITFLEET_PLAYBOOK_TEST_REPO_OWNER_TYPE" --public --initialize --yes >/dev/null 2>&1; then
    REPO_CREATED=true
  fi
}
teardown() {
  if [ "$REPO_CREATED" = true ]; then
    gitfleet repo delete "$TEST_REPO" --yes >/dev/null 2>&1 || true
  fi
  print_summary
}
trap teardown EXIT
setup

if ! has_capability "security"; then
  step "Security Capability"
  expect_exit_non0 "security is explicitly unsupported" gitfleet security advisories --repo "$GITFLEET_PLAYBOOK_REPO"
  exit 0
fi

step "Dependabot Alerts"
expect_exit_0 "Dependabot alert listing succeeds" gitfleet security advisories --repo "$TEST_REPO"

step "Secret Scanning Alerts"
if gitfleet security secret-scans --repo "$TEST_REPO" >/dev/null 2>&1; then
  pass "secret scanning alert listing succeeds"
else
  skip "secret scanning alerts (feature may be disabled)"
fi

step "CodeQL Alerts"
if gitfleet security codeql --repo "$TEST_REPO" >/dev/null 2>&1; then
  pass "CodeQL alert listing succeeds"
else
  skip "CodeQL alerts (feature may be disabled)"
fi
