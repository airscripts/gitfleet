#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

MILESTONE_NUMBER=""

setup() { :; }

teardown() {
  if [ -n "$MILESTONE_NUMBER" ]; then
    gitfleet planning milestone delete "$MILESTONE_NUMBER" --repo "$REPO" --yes >/dev/null 2>&1 || true
  fi
  print_summary
}

trap teardown EXIT
setup

step "Milestone List"
expect_exit_0 "milestone list succeeds" gitfleet planning milestone list --repo "$REPO"

step "Milestone Create"
output=$(gitfleet planning milestone create "gitfleet-test-milestone-$PB_RESOURCE_SUFFIX" --repo "$REPO" --json 2>&1) || true
MILESTONE_NUMBER=$(echo "$output" | python3 -c "import sys,json; print(json.load(sys.stdin).get('number',''))" 2>/dev/null || echo "")

if [ -n "$MILESTONE_NUMBER" ]; then
  pass "milestone create succeeded (number=$MILESTONE_NUMBER)"
else
  fail "milestone create failed"
fi

step "Milestone View"
if [ -n "$MILESTONE_NUMBER" ]; then
  expect_exit_0 "milestone view succeeds" gitfleet planning milestone view "$MILESTONE_NUMBER" --repo "$REPO"
else
  skip "milestone view (create failed)"
fi

step "Milestone Update"
if [ -n "$MILESTONE_NUMBER" ]; then
  expect_exit_0 "milestone update succeeds" gitfleet planning milestone update "$MILESTONE_NUMBER" --repo "$REPO" --description "Updated by gitfleet playbook"
else
  skip "milestone update (create failed)"
fi

step "Milestone Delete"
if [ -n "$MILESTONE_NUMBER" ]; then
  expect_exit_0 "milestone delete succeeds" gitfleet planning milestone delete "$MILESTONE_NUMBER" --repo "$REPO" --yes
  MILESTONE_NUMBER=""
else
  skip "milestone delete (create failed)"
fi
