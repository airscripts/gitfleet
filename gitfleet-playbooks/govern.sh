#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

setup() { :; }
RULESET_ID=""
TEST_REPO_NAME="gitfleet-test-govern-$GITFLEET_PLAYBOOK_RESOURCE_SUFFIX"
TEST_REPO="$GITFLEET_PLAYBOOK_TEST_REPO_OWNER/$TEST_REPO_NAME"
REPO_CREATED=false

teardown() {
  if [ -n "$RULESET_ID" ]; then
    gitfleet govern delete-ruleset "$RULESET_ID" --repo "$TEST_REPO" --yes >/dev/null 2>&1 || true
  fi
  if [ "$REPO_CREATED" = true ]; then
    gitfleet repo delete "$TEST_REPO" --yes >/dev/null 2>&1 || true
  fi
  print_summary
}
trap teardown EXIT
setup

step "Govern List-Rulesets"
if ! has_capability "governance"; then
  expect_exit_non0 "governance is explicitly unsupported" gitfleet govern list-rulesets --repo "$GITFLEET_PLAYBOOK_REPO"
  exit 0
fi

if gitfleet repo create "$TEST_REPO_NAME" --owner "$GITFLEET_PLAYBOOK_TEST_REPO_OWNER" --owner-type "$GITFLEET_PLAYBOOK_TEST_REPO_OWNER_TYPE" --public --initialize --yes >/dev/null 2>&1; then
  REPO_CREATED=true
else
  fail "governance test repository creation failed"
fi

expect_exit_0 "govern list-rulesets succeeds" gitfleet govern list-rulesets --repo "$TEST_REPO"

step "Govern Create Ruleset"
output=$(gitfleet govern create-ruleset --name "gitfleet-test-$GITFLEET_PLAYBOOK_RESOURCE_SUFFIX" --repo "$TEST_REPO" --json 2>&1) || true
RULESET_ID=$(echo "$output" | python3 -c 'import sys,json; print(json.load(sys.stdin).get("id",""))' 2>/dev/null || echo "")

if [ -n "$RULESET_ID" ]; then
  pass "govern create-ruleset succeeds"
else
  fail "govern create-ruleset did not return an id"
fi

step "Govern Delete Ruleset"
if [ -n "$RULESET_ID" ]; then
  expect_exit_0 "govern delete-ruleset succeeds" gitfleet govern delete-ruleset "$RULESET_ID" --repo "$TEST_REPO" --yes
  RULESET_ID=""
else
  fail "govern delete-ruleset has no ruleset id"
fi
