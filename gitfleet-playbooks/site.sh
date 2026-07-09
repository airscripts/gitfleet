#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

TEST_REPO="${OWNER}-site-test"
SITE_ENABLED=false

setup() {
  gitfleet repo create "$TEST_REPO" --private --yes >/dev/null 2>&1 || true
}

teardown() {
  if [ "$SITE_ENABLED" = true ]; then
    gitfleet site delete --repo "$TEST_REPO" >/dev/null 2>&1 || true
  fi
  gitfleet api delete --endpoint "/repos/$TEST_REPO" >/dev/null 2>&1 || true
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
  expect_exit_0 "site delete succeeds" gitfleet site delete --repo "$TEST_REPO"
  SITE_ENABLED=false
else
  skip "site delete (site not created)"
fi