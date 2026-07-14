#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

TEST_REPO_NAME="gitfleet-test-site-$GITFLEET_PLAYBOOK_RESOURCE_SUFFIX"
TEST_REPO="$GITFLEET_PLAYBOOK_TEST_REPO_OWNER/$TEST_REPO_NAME"
REPO_CREATED=false
SITE_ENABLED=false

setup() {
  if gitfleet repo create "$TEST_REPO_NAME" --owner "$GITFLEET_PLAYBOOK_TEST_REPO_OWNER" --owner-type "$GITFLEET_PLAYBOOK_TEST_REPO_OWNER_TYPE" --public --initialize --yes >/dev/null 2>&1; then
    REPO_CREATED=true
  else
    fail "site test repository creation failed"
  fi
}

teardown() {
  if [ "$SITE_ENABLED" = true ]; then
    gitfleet site delete --repo "$TEST_REPO" --yes >/dev/null 2>&1 || true
  fi
  if [ "$REPO_CREATED" = true ]; then
    gitfleet repo delete "$TEST_REPO" --yes >/dev/null 2>&1 || true
  fi
  print_summary
}

trap teardown EXIT

if ! has_capability "site"; then
  step "Site Capability"
  expect_exit_non0 "sites are explicitly unsupported" gitfleet site get --repo "$GITFLEET_PLAYBOOK_REPO"
  exit 0
fi

setup

step "Site Get (before enable)"
if gitfleet site get --repo "$TEST_REPO" >/dev/null 2>&1; then
  pass "site get succeeds"
else
  pass "site get reports no configured site"
fi

step "Site Create"
if gitfleet site create --repo "$TEST_REPO" --source main >/dev/null 2>&1; then
  pass "site create succeeded"
  SITE_ENABLED=true
else
  fail "site create failed"
fi

step "Site Get (after create)"
if [ "$SITE_ENABLED" = true ]; then
  expect_exit_0 "site get after create succeeds" gitfleet site get --repo "$TEST_REPO"
else
  skip "site get after create (site not created)"
fi

step "Site Delete"
if [ "$SITE_ENABLED" = true ]; then
  expect_exit_0 "site delete succeeds" gitfleet site delete --repo "$TEST_REPO" --yes
  SITE_ENABLED=false
else
  skip "site delete (site not created)"
fi
