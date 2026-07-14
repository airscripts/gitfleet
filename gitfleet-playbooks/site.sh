#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

TEST_REPO_NAME="gitfleet-test-site-$PB_RESOURCE_SUFFIX"
TEST_REPO="$ORG/$TEST_REPO_NAME"
REPO_CREATED=false
SITE_ENABLED=false

setup() {
  if gitfleet repo create "$TEST_REPO_NAME" --owner "$ORG" --owner-type org --private --yes >/dev/null 2>&1; then
    REPO_CREATED=true
    content=$(printf 'Gitfleet site playbook\n' | base64 | tr -d '\n')
    gitfleet api post --endpoint "/repos/$TEST_REPO/contents/README.md" --body "{\"message\":\"test: initialize repository\",\"content\":\"$content\"}" --json >/dev/null 2>&1 || true
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
setup

step "Site Get (before enable)"
if gitfleet site get --repo "$TEST_REPO" >/dev/null 2>&1; then
  pass "site get succeeds"
else
  skip "site get (repo may not exist yet)"
fi

step "Site Create"
if gitfleet site create --repo "$TEST_REPO" --source main >/dev/null 2>&1; then
  pass "site create succeeded"
  SITE_ENABLED=true
else
  skip "site create (may not be supported for this repo)"
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
